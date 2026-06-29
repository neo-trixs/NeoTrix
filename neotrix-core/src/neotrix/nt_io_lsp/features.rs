use std::path::Path;

use super::{CompletionItem, Diagnostic, HoverInfo, Location, LspClient, LspError, Position};

pub struct LspFeatures {
    client: LspClient,
}

impl LspFeatures {
    pub fn new(client: LspClient) -> Self {
        Self { client }
    }

    pub fn into_inner(self) -> LspClient {
        self.client
    }

    pub fn client(&mut self) -> &mut LspClient {
        &mut self.client
    }

    pub async fn autocomplete(
        &mut self,
        point: Position,
        file: &Path,
    ) -> Result<Vec<CompletionItem>, LspError> {
        self.client.completion(file, point).await
    }

    pub async fn hover(
        &mut self,
        point: Position,
        file: &Path,
    ) -> Result<Option<HoverInfo>, LspError> {
        self.client.hover(file, point).await
    }

    pub async fn goto_definition(
        &mut self,
        point: Position,
        file: &Path,
    ) -> Result<Option<Location>, LspError> {
        self.client.goto_definition(file, point).await
    }

    pub async fn find_references(
        &mut self,
        point: Position,
        file: &Path,
    ) -> Result<Vec<Location>, LspError> {
        self.client.references(file, point).await
    }

    pub async fn diagnostics(&mut self, file: &Path) -> Result<Vec<Diagnostic>, LspError> {
        self.client.diagnostics(file).await
    }

    pub async fn format(&mut self, file: &Path) -> Result<String, LspError> {
        self.client.format(file).await
    }
}
