use crate::{pkg::{internal::repo::create_new_repo, server::state::AppState}, prelude::Result};
use serde::Serialize;
use sqlx::{prelude::{FromRow, Type}, PgConnection, Postgres, QueryBuilder, Transaction};
use standard_error::StandardError;
use uuid::Uuid;

use super::{auth::User, email::invite::ShowInvite, repo::{self, delete_repo}};

#[derive(Debug, Type)]
#[sqlx(type_name = "invite_status", rename_all = "lowercase")]
pub enum InviteStatus {
    Pending,
    Accepted,
    Expired,
}

#[derive(FromRow, Serialize, Debug, Clone)]
pub struct Project {
    pub project_id: String,
    pub name: String,
    pub description: String,
}

#[derive(FromRow, Debug)]
pub struct AccessInvite {
    pub user_id: String,
    pub invite_id: String,
    pub project_id: String,
    pub inviter_id: String,
    pub status: InviteStatus
}

impl Project {
    pub async fn create(state: &AppState, name: &str, description: &str, user_id: &str) -> Result<Self> {
        let mut tx = state.db_pool.begin().await?;
        let txn = &mut *tx;
        let project = sqlx::query_as!(
            Project,
            r#"
            INSERT INTO projects (name, description, project_id)
            VALUES ($1, $2, $3)
            ON CONFLICT (name) DO NOTHING
            RETURNING project_id, name, description
            "#,
            name,
            description,
            Uuid::new_v4().to_string()
        )
        .fetch_one(&mut *txn)
        .await?;
   
        let projclone = project.clone();
        let invite_result = async move {
            projclone.invite(txn, &user_id, &user_id).await?;
            Ok::<(), StandardError>(())
        };
  
        let repo_details = project.clone();
        let repo_result = async move {
            create_new_repo(&repo_details.name, &repo_details.description).await?;
            Ok::<(), StandardError>(())
        };
    
        let (invite_res, repo_res) = tokio::join!(invite_result, repo_result);
    
        match (invite_res, repo_res) {
            (Ok(()), Ok(())) => {
                tx.commit().await?;
                Ok(project)
            }
            (Err(e), _) | (_, Err(e)) => {
                tx.rollback().await?;
                delete_repo(&project.name).await?;
                Err(e)
            }
        }
    }
    
    pub async fn list(state: &AppState, user_id: &str) -> Result<Vec<Self>> {
        let project_ids: Vec<String> = sqlx::query_scalar!(
            "select project_id from project_access where status = $2 and user_id = $1",
            &user_id,
            InviteStatus::Accepted as _
        )
        .fetch_all(&*state.db_pool)
        .await?;
        if project_ids.is_empty(){
            return Ok(vec![])
        }
        let mut qb = QueryBuilder::new("select project_id, name, description from projects where project_id in (");
        qb.push_bind(&project_ids[0]);
        for pid in &project_ids[1..] {
            qb.push(", ").push_bind(pid);
        }
        qb.push(")");
        let projects = qb
            .build_query_as::<Project>()
            .fetch_all(&*state.db_pool)
            .await?;
        Ok(projects)
    }

    pub async fn retrieve(state: &AppState, project_id: &str) -> Result<Self> {
        let project = sqlx::query_as!(
            Project,
            "select project_id, name, description from projects where project_id = $1",
            &project_id
        )
        .fetch_one(&*state.db_pool)
        .await?;
        Ok(project)
    }

    pub async fn delete(&self, state: &AppState) -> Result<()> {
        sqlx::query!(
            "delete from project_access where project_id = $1",
            &self.project_id
        )
        .execute(&*state.db_pool)
        .await?;
        sqlx::query!(
            "delete from projects where project_id = $1",
            &self.project_id
        )
        .execute(&*state.db_pool)
        .await?;
        Ok(())
    }

    pub async fn invite(&self, txn: &mut PgConnection, user_id: &str, me: &str) -> Result<AccessInvite> {
        let invite = sqlx::query_as!(
            AccessInvite,
            r#"
            insert into project_access (invite_id, project_id, user_id, expiry, inviter_id)
            values ($1, $2, $3, NOW() + interval '1 hour', $4)
            on conflict (project_id, user_id) do update 
            set expiry = NOW() + INTERVAL '1 hour'
            returning user_id, invite_id, project_id, inviter_id, status as "status:_"
            "#,
            Uuid::new_v4().to_string(),
            &self.project_id,
            user_id,
            &me
        )
        .fetch_one(&mut *txn)
        .await?;
        if user_id == me{
            invite.accept(txn).await?;
        }
        Ok(invite)
    }
}

impl AccessInvite {
    pub async fn new(state: &AppState, invite_id: &str) -> Result<Self> {
        Ok(sqlx::query_as!(
            Self,
            r#"select user_id, project_id, invite_id, inviter_id, status as "status:_" from project_access
            where invite_id = $1"#,
            &invite_id
        )
        .fetch_one(&*state.db_pool)
        .await?)
    }

    pub async fn details(&self, state: &AppState) -> Result<ShowInvite> {
        let project = sqlx::query_as!(
            Project,
            "select project_id, name, description from projects where project_id = $1",
            &self.project_id
        )
        .fetch_one(&*state.db_pool)
        .await?;
        let inviter = sqlx::query_scalar!(
            "select name from users where user_id = $1",
            &self.inviter_id
        )
        .fetch_one(&*state.db_pool)
        .await?;
        Ok(ShowInvite {
            invite_id: self.invite_id.to_string(),
            inviter: inviter.to_string(),
            project_name: project.name,
            project_description: project.description,
        })
    }

    pub async fn accept(&self, txn: &mut PgConnection) -> Result<()> {
        match sqlx::query_as!(
            Self,
            r#"
            update project_access set status = $2 where invite_id = $1 and expiry > NOW()
            returning user_id, invite_id, project_id, inviter_id, status as "status:_"
            "#,
            &self.invite_id,
            InviteStatus::Accepted as _
        )
        .fetch_optional(txn)
        .await?
        {
            Some(_) => {}
            None => {
                return Err(StandardError::new("ERR-INVITE-EXPIRED"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::pkg::internal::auth::User;
    use tracing_test::traced_test;

    use super::*;

    #[tokio::test]
    #[traced_test]
    async fn test_project_crud() -> Result<()> {
        let state = AppState::new().await?;
        let user = User::create(&state, "a@a.com", "a").await?;
        let before = Project::list(&state, &user.user_id).await?.len();
        Project::create(&state, "proj1", "first project", &user.user_id).await?;
        Project::create(&state, "proj2", "second project", &user.user_id).await?;
        Project::create(&state, "proj3", "third project", &user.user_id).await?;
        let projects = Project::list(&state, &user.user_id).await?;
        assert_eq!(projects.len() - before, 3);
        let project_id = projects[0].project_id.clone();
        let project = Project::retrieve(&state, &project_id).await?;
        project.delete(&state).await?;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    async fn test_project_invite_accept() -> Result<()> {
        let state = AppState::new().await?;
        let user = User::create(&state, "ashupednekar49@gmail.com", "Ashu Pednekar").await?;
        Project::create(&state, "proj", "project description", &user.user_id).await?;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    async fn test_project_invite_expiry() -> Result<()> {
        Ok(())
    }
}
