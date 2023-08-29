//! Query cache.

use crate::{
    errors::Error,
    plugins::{Plugin, PluginState, PluginOutput},
    query_router::QueryRouter, messages::{row_description, DataType, data_row, command_complete},
};
use async_trait::async_trait;
use sqlparser::ast::Statement;
use std::collections::HashMap;
use bytes::BufMut;
use bytes::BytesMut;

#[derive(Debug)]
pub struct QueryCacheImpl {
    explain_analyze_cache: HashMap<String, Vec<BytesMut>>,
}

pub struct QueryCache<'a> {
    pub enabled: bool,
    pub user: &'a str,
    pub db: &'a str,
}

#[async_trait]
impl<'a> Plugin for QueryCache<'a> {
    async fn run(
        &mut self,
        _query_router: &QueryRouter,
        plugin_state: &mut PluginState,
        ast: &Vec<Statement>,
    ) -> Result<PluginOutput, Error> {
        if !self.enabled || ast.is_empty() {
            return Ok(PluginOutput::Allow);
        }

        let mut ea_cache = &plugin_state.query_cache.explain_analyze_cache;
        let mut result = BytesMut::new();

        if ast.len() > 1 {
            return Ok(PluginOutput::Allow)
        }
        let query = ast[0].to_string().to_ascii_lowercase();

        if let Some(response) = ea_cache.get(&query) {
            let rd = vec![("QUERY PLAN", DataType::Text)];
            let response = vec![response.clone()];

            result.put(row_description(&rd));
            result.put(data_row(&response));
            result.put(command_complete("EXPLAIN"));
        }

        if !result.is_empty() {
            result.put_u8(b'Z');
            result.put_i32(5);
            result.put_u8(b'I');
            Ok(PluginOutput::Intercept(result))
        } else {
            Ok(PluginOutput::Allow)
        }
    }

    async fn run_post(
        &mut self,
        query_router: &QueryRouter,
        plugin_state: &mut PluginState,
        ast: &Vec<Statement>,
        responses: &Vec<Vec<BytesMut>>,
    ) {
        let it = ast.iter().zip(responses.iter());
        let mut cache = plugin_state.query_cache.explain_analyze_cache;

        for (query, response) in it {
            cache.insert(query.to_string().to_ascii_lowercase(), response.clone());
        }

    }
}

impl QueryCacheImpl {
    pub fn new() -> Self {
        QueryCacheImpl { 
            explain_analyze_cache: HashMap::new(),
        }
    }
}