// Copyright 2021 The Engula Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod block_builder;
mod block_handle;
mod block_reader;
mod block_scanner;
mod table_builder;
mod table_reader;
mod table_scanner;

use std::sync::Arc;

use engula_futures::io::RandomRead;

pub type RandomReader = Arc<dyn RandomRead + Send + Sync + Unpin>;

pub use self::{
    table_builder::{TableBuilder, TableBuilderOptions},
    table_reader::TableReader,
    table_scanner::TableScanner,
};
