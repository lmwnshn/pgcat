//! Log all queries to stdout (or somewhere else, why not).

use crate::{
    errors::Error,
    plugins::{Plugin, PluginState, PluginOutput},
    query_router::QueryRouter,
};
use async_trait::async_trait;
use bytes::BytesMut;
use log::info;
use sqlparser::ast::Statement;

pub struct QueryLogger<'a> {
    pub enabled: bool,
    pub user: &'a str,
    pub db: &'a str,
}

#[async_trait]
impl<'a> Plugin for QueryLogger<'a> {
    async fn run(
        &mut self,
        _query_router: &QueryRouter,
        _plugin_state: &mut PluginState,
        ast: &Vec<Statement>,
    ) -> Result<PluginOutput, Error> {
        if !self.enabled {
            return Ok(PluginOutput::Allow);
        }

        let query = ast
            .iter()
            .map(|q| q.to_string())
            .collect::<Vec<String>>()
            .join("; ");
        info!("[pool: {}][user: {}] {}", self.user, self.db, query);

        Ok(PluginOutput::Allow)
    }

    async fn run_post(
        &mut self,
        query_router: &QueryRouter,
        plugin_state: &mut PluginState,
        ast: &Vec<Statement>,
        responses: &Vec<Vec<BytesMut>>,
    ) {}
}
