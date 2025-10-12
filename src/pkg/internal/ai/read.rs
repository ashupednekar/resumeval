use std::io::Cursor;
use crate::prelude::Result;
use standard_error::{Interpolate, StandardError};


pub fn extract_document(data: Vec<u8>, content_type: &str) -> Result<String>{
   match content_type {
       "application/pdf" => extract_text_from_pdf(&data),
       "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => {
           extract_text_from_docx(&data)
       }
       "application/msword" => extract_text_from_doc(&data),
       "text/plain" => Ok(String::from_utf8(data.clone())
           .map_err(|e| StandardError::new("ERR-AI-005").interpolate_err(e.to_string()))?),
       _ => return Err(StandardError::new("ERR-AI-005")),
   }
}

fn extract_text_from_pdf(data: &[u8]) -> Result<String> {
    use lopdf::Document;
    let cursor = Cursor::new(data);
    let doc = Document::load_from(cursor)
        .map_err(|e| StandardError::new("ERR-AI-005").interpolate_err(e.to_string()))?;
    
    let pages = doc.get_pages();
    let mut text = String::new();
    
    for page_num in pages.keys() {
        match doc.extract_text(&[*page_num]) {
            Ok(page_text) => {
                text.push_str(&page_text);
                text.push(' ');
            }
            Err(e) => {
                eprintln!("Warning: Failed to extract text from page {}: {}", page_num, e);
            }
        }
    }
    
    if text.trim().is_empty() {
        return Err(StandardError::new("ERR-AI-005")
            .interpolate_err("No text extracted from PDF".to_string()));
    }
    Ok(text.trim().to_string())
}


fn extract_text_from_docx(data: &[u8]) -> Result<String> {
    use docx_rs::read_docx;
    let docx = read_docx(&data)
        .map_err(|e| StandardError::new("ERR-AI-005"))?;
    let mut text = String::new();
    for paragraph in docx.document.children {
        if let docx_rs::DocumentChild::Paragraph(p) = paragraph {
            for child in p.children {
                if let docx_rs::ParagraphChild::Run(run) = child {
                    for run_child in run.children {
                        if let docx_rs::RunChild::Text(t) = run_child {
                            text.push_str(&t.text);
                        }
                    }
                }
            }
            text.push('\n');
        }
    }
    Ok(text)
}

fn extract_text_from_doc(_data: &[u8]) -> Result<String> {
    Err(StandardError::new("ERR-AI-005"))
}
