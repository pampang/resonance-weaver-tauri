use lancedb::connection::Connection;
use lancedb::table::Table;
use lancedb::connect;
use arrow_array::{Float32Array, StringArray, RecordBatch, Int64Array, ArrayRef, FixedSizeListArray};
use arrow_schema::{DataType, Field, Schema};
use lancedb::query::{ExecutableQuery, QueryBase};
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use tauri::AppHandle;
use tauri::Manager;
use futures::StreamExt;

#[derive(Serialize, Deserialize, Debug)]
struct OllamaEmbeddingResponse {
    embedding: Vec<f32>,
}

#[derive(Serialize, Deserialize, Debug)]
struct OllamaEmbeddingRequest {
    model: String,
    prompt: String,
}

pub struct VectorStore {
    pub table: Table,
}

impl VectorStore {
    pub async fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let app_dir = app_handle.path().app_data_dir().expect("failed to get app data dir");
        let db_path = app_dir.join("lancedb");
        if !db_path.exists() {
            std::fs::create_dir_all(&db_path).map_err(|e| e.to_string())?;
        }

        let uri = db_path.to_str().ok_or("Invalid path")?;
        let db = connect(uri).execute().await.map_err(|e| e.to_string())?;

        let table_name = "resonance";
        let table = if db.table_names().execute().await.map_err(|e| e.to_string())?.contains(&table_name.to_string()) {
            db.open_table(table_name).execute().await.map_err(|e| e.to_string())?
        } else {
            Self::create_empty_table(&db, table_name).await?
        };

        Ok(Self { table })
    }

    async fn create_empty_table(db: &Connection, name: &str) -> Result<Table, String> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("text", DataType::Utf8, false),
            Field::new("vector", DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, true)), 768), false),
            Field::new("metadata", DataType::Utf8, true),
        ]));

        let id_array = Int64Array::from(vec![0]);
        let text_array = StringArray::from(vec!["dummy"]);
        let metadata_array = StringArray::from(vec![None as Option<&str>]);
        
        let vector_values = Float32Array::from(vec![0.0f32; 768]);
        let vector_array = FixedSizeListArray::try_new(
            Arc::new(Field::new("item", DataType::Float32, true)),
            768,
            Arc::new(vector_values),
            None
        ).map_err(|e| e.to_string())?;

        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(id_array) as ArrayRef,
                Arc::new(text_array) as ArrayRef,
                Arc::new(vector_array) as ArrayRef,
                Arc::new(metadata_array) as ArrayRef,
            ],
        ).map_err(|e| e.to_string())?;

        db.create_table(name, vec![batch])
            .execute()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn get_embedding(text: &str) -> Result<Vec<f32>, String> {
        let client = Client::new();
        let res = client.post("http://localhost:11434/api/embeddings")
            .json(&OllamaEmbeddingRequest {
                model: "nomic-embed-text".to_string(),
                prompt: text.to_string(),
            })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let body: OllamaEmbeddingResponse = res.json().await.map_err(|e| e.to_string())?;
        Ok(body.embedding)
    }

    pub async fn add_resonance(&self, id: i64, text: String, embedding: Vec<f32>, metadata: Option<String>) -> Result<(), String> {
        let schema = self.table.schema().await.map_err(|e| e.to_string())?;
        
        let id_array = Int64Array::from(vec![id]);
        let text_array = StringArray::from(vec![text]);
        let metadata_array = StringArray::from(vec![metadata]);
        
        let vector_values = Float32Array::from(embedding);
        let vector_array = FixedSizeListArray::try_new(
            Arc::new(Field::new("item", DataType::Float32, true)),
            768,
            Arc::new(vector_values),
            None
        ).map_err(|e| e.to_string())?;

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_array) as ArrayRef,
                Arc::new(text_array) as ArrayRef,
                Arc::new(vector_array) as ArrayRef,
                Arc::new(metadata_array) as ArrayRef,
            ],
        ).map_err(|e| e.to_string())?;

        self.table.add(vec![batch]).execute().await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub async fn search(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<(String, f32)>, String> {
        let query = self.table.query();
        let query = query.nearest_to(vector).map_err(|e| e.to_string())?.limit(limit);
        
        let mut results = query.execute().await.map_err(|e| e.to_string())?;

        let mut matches = Vec::new();
        while let Some(batch) = results.next().await {
            let batch = batch.map_err(|e| e.to_string())?;
            let text_array = batch.column_by_name("text")
                .ok_or("text column not found")?
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or("failed to downcast text column")?;
            
            let distance_array = batch.column_by_name("_distance");

            for i in 0..batch.num_rows() {
                let text = text_array.value(i).to_string();
                let distance = if let Some(dist_col) = distance_array {
                    dist_col.as_any().downcast_ref::<Float32Array>()
                        .map(|a| a.value(i))
                        .unwrap_or(0.0)
                } else {
                    0.0
                };
                if text != "dummy" {
                    matches.push((text, distance));
                }
            }
        }

        Ok(matches)
    }
}
