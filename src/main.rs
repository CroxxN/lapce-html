use anyhow::{anyhow, Result};
use lapce_plugin::{
  psp_types::{
    lsp_types::{request::Initialize, DocumentFilter, DocumentSelector, InitializeParams, Url},
    Request,
  },
  register_plugin, LapcePlugin, PLUGIN_RPC,
};
use serde_json::Value;

#[derive(Default)]
struct State {}

register_plugin!(State);

macro_rules! string {
  ( $x:expr ) => {
    String::from($x)
  };
}

macro_rules! ok {
  ( $x:expr ) => {
    match ($x) {
      | Ok(v) => v,
      | Err(e) => return Err(anyhow!(e)),
    }
  };
}

fn initialize(params: InitializeParams) -> Result<()> {
  let document_selector: DocumentSelector = vec![DocumentFilter {
    language: Some(string!("html")),
    pattern: Some(string!("**/*.{html,htm,js,ts,css,jsx,tsx}")),
    scheme: None,
  }];
  let mut server_args = vec![string!("--stdio")];

  if let Some(options) = params.initialization_options.as_ref() {
    if let Some(volt) = options.get("volt") {
      if let Some(args) = volt.get("serverArgs") {
        if let Some(args) = args.as_array() {
          if !args.is_empty() {
            server_args = vec![];
          }
          for arg in args {
            if let Some(arg) = arg.as_str() {
              server_args.push(string!(arg));
            }
          }
        }
      }

      if let Some(server_path) = volt.get("serverPath") {
        if let Some(server_path) = server_path.as_str() {
          if !server_path.is_empty() {
            let server_uri = ok!(Url::parse(&format!("urn:{}", server_path)));
            PLUGIN_RPC.start_lsp(
              server_uri,
              server_args,
              document_selector,
              params.initialization_options,
            );
            return Ok(());
          }
        }
      }
    }
  }

  let server_uri = ok!(Url::parse("urn:@kozer/emmet-language-server"));

  PLUGIN_RPC.start_lsp(
    server_uri,
    server_args,
    document_selector,
    params.initialization_options,
  );

  Ok(())
}

impl LapcePlugin for State {
  fn handle_request(&mut self, _id: u64, method: String, params: Value) {
    #[allow(clippy::single_match)]
    match method.as_str() {
      | Initialize::METHOD => {
        let params: InitializeParams = serde_json::from_value(params).unwrap();
        let _ = initialize(params);
      }
      | _ => {}
    }
  }
}