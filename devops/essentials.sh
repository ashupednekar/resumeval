helm install pgo oci://registry.developers.crunchydata.com/crunchydata/pgo
helm install minio oci://registry-1.docker.io/bitnamicharts/minio -f minio.yaml
kubectl apply -f dbcluster.yaml

