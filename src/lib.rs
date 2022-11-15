use std::{error::Error, ops::DerefMut};

use reqwest::{Client, header::HeaderMap};
use serde::{Serialize, Deserialize};
use serde_json::{Value, Map};
use serde_this_or_that::as_string;

mod test;

pub struct Godbolt {
    api_client: Client,
}

impl Godbolt {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", "application/json".parse().expect("Error inserting headers."));
        
        let client = Client::builder()  
            .default_headers(headers)
            .build().unwrap();

        Self {
            api_client: client
        }
    }

    /// GET /api/compilers - return a list of compilers
    pub async fn get_compilers(&self) -> Result<Vec<Compiler>, Box<dyn Error>> {
        Ok(
            self.api_client.get("https://godbolt.org/api/compilers/")
                .send()
                .await?
                .json::<Vec<Compiler>>()
                .await?
        )
    }
    /// GET /api/compilers?fields - return a list of compilers with added fields
    /// If you require different fields, you can specify them by adding a list of fields to your query.
    pub async fn get_compilers_with_fields(&self, fields: &[&str]) -> Result<Vec<Compiler>, Box<dyn Error>> {
        let additional_fields = fields.join(",");
        let response = self.api_client.get(format!("https://godbolt.org/api/compilers?fields=compilerType,id,instructionSet,lang,name,semver,{}", additional_fields))
            .send()
            .await?
            .text()
            .await?;
       
        Ok(
            serde_json::from_str::<Vec<Compiler>>(&response)?
                .iter_mut()
                .zip(serde_json::from_str::<Vec<AdditionalFields>>(&response)?)
                .map(|(mut comp, field)| {
                    comp.additional_fields = Some(field.clone());
                    comp.deref_mut().clone()
                }).collect::<Vec<Compiler>>()
        )
    }
    /// GET /api/compilers?fields=all - return a list of compilers with all fields
    /// If you require all fields of a compiler use this function
    pub async fn get_compilers_with_all_fields(&self) -> Result<Vec<Compiler>, Box<dyn Error>> {
        let response = self.api_client.get("https://godbolt.org/api/compilers?fields=all")
            .send()
            .await?
            .text()
            .await?;

        Ok(
            serde_json::from_str::<Vec<Compiler>>(&response)?
                .iter_mut()
                .zip(serde_json::from_str::<Vec<AdditionalFields>>(&response)?)
                .map(|(mut comp, field)| {
                    comp.additional_fields = Some(field.clone());
                    comp.deref_mut().clone()
                }).collect::<Vec<Compiler>>()
        )
    }

    /// GET /api/compilers/{language-id} - return a list of compilers with a matching language
    pub async fn get_compiler_by_id(&self, language: &str) -> Result<Vec<Compiler>, Box<dyn Error>> {
        Ok(
            self.api_client.get(format!("https://godbolt.org/api/compilers/{}", language))
                .send()
                .await?
                .json::<Vec<Compiler>>()
                .await?        
        )
    }

    /// GET /api/languages - return a list of languages
    pub async fn get_languages(&self) -> Result<Vec<Language>, Box<dyn Error>> {
        Ok(
            self.api_client.get("https://godbolt.org/api/languages/")
                .send()
                .await?
                .json::<Vec<Language>>()
                .await?
        )
    }

    /// GET /api/libraries/{language-id} - return a list of libraries available with for a language
    pub async fn get_library_by_id(&self, language: &str) -> Result<Vec<Library>, Box<dyn Error>> {
        Ok(
            self.api_client.get(format!("https://godbolt.org/api/libraries/{}", language))
                .send()
                .await?
                .json::<Vec<Library>>()
                .await?
        )
    }

    /// GET /api/formats - return available code formatters
    pub async fn get_formats(&self) -> Result<Vec<Format>, Box<dyn Error>> {
        Ok(
            self.api_client.get("https://godbolt.org/api/formats")
                .send()
                .await?
                .json::<Vec<Format>>()
                .await?
        )
    }

    /// POST /api/compiler/{compiler-id}/compile - perform a compilation
    pub async fn send_request(&self, compiler: &str, source: &str) -> Result<CompilationResponse, Box<dyn Error>> {
        let filters = Filters::default();
        let options = Options{user_arguments: "-O".into(), filters};
        let source = Source{source: source.into(), options};
        
        Ok(
            self.api_client.post(format!("https://godbolt.org/api/compiler/{}/compile", compiler))
                .json(&source)
                .send()
                .await?
                .json::<CompilationResponse>()
                .await?
        )
    }

    // POST /api/compiler/{compiler-id}/compile - perform a compilation
    /// Send a request with additional compilation options
    pub async fn send_request_with_options(&self, compiler: &str, source: &str, options: Options) -> Result<CompilationResponse, Box<dyn Error>> {
        let source = Source{source: source.into(), options};
        
        Ok(
            self.api_client.post(format!("https://godbolt.org/api/compiler/{}/compile", compiler))
                .json(&source)
                .send()
                .await?
                .json::<CompilationResponse>()
                .await?
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct CompilationResponse {
    input_filename: String,
    code: i64,
    ok_to_cache: bool,
    timed_out: bool,
    stdout: Vec<Option<serde_json::Value>>,
    stderr: Vec<Option<serde_json::Value>>,
    exec_time: String,
    compilation_options: Vec<String>,
    downloads: Vec<Option<serde_json::Value>>,
    tools: Vec<Option<serde_json::Value>>,
    asm_size: i64,
    asm: Vec<Value>,
    label_definitions: Value,
    parsing_time: String,
    filtered_count: i64,
    popular_arguments: Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Source {
    source: String,
    options: Options,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Options {
    user_arguments: String,
    filters: Filters,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Filters {
    binary: bool,
    comment_only: bool,
    demangle: bool,
    directives: bool,
    execute: bool,
    intel: bool,
    labels: bool,
    library_code: bool,
    trim: bool,
}

impl Default for Filters {
    fn default() -> Self {
        Self { binary: false, comment_only: true, demangle: true, directives: true, execute: false, intel: true, labels: true, library_code: false, trim: false }
    }
}

#[derive(Debug, Deserialize)]
pub struct Format {
    exe: String,
    name: String,
    styles: Vec<String>,
    #[serde(rename(deserialize = "type"))]
    format_type: String,
    version: String,
}

#[derive(Debug, Deserialize)]
pub struct Library {
    id: String,
    name: String,
    url: String,
    versions: Vec<Map<String, Value>>,
}

#[derive(Debug, Deserialize)]
pub struct Language {
    id: String,
    name: String,
    extensions: Vec<String>,
    monaco: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct Compiler {
    compiler_type: String,
    id: String,
    #[serde(deserialize_with = "as_string")]
    instruction_set: String,
    lang: String,
    name: String,
    semver: String,
    additional_fields: Option<AdditionalFields>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all="camelCase")]
pub struct AdditionalFields {
    adarts: Option<String>,
    alias: Option<Vec<String>>,
    #[serde(deserialize_with = "as_string")]
    #[serde(default="default_string")]
    demangler: String,
    demangler_type: Option<String>,
    disabled_filters: Option<Vec<String>>,
    env_vars: Option<Vec<Value>>,
    #[serde(deserialize_with = "as_string")]
    #[serde(default="default_string")]
    exe: String,
    execution_wrapper: Option<String>,
    full_version: Option<String>,
    group: Option<String>,
    group_name: Option<String>,
    hidden: Option<bool>,
    id: Option<String>,
    include_flag: Option<String>,
    include_path: Option<String>,
    intel_asm: Option<String>,
    interpreted: Option<bool>,
    is_sem_ver: Option<bool>,
    ld_path: Option<Vec<String>>,
    lib_path: Option<Vec<String>>,
    #[serde(rename="libpathFlag")]
    lib_path_flag: Option<String>,
    libs_arr: Option<Vec<String>>,
    license: Option<Map<String, Value>>,
    link_flag: Option<String>,
    needs_multi: Option<bool>,
    notification: Option<String>,
    #[serde(default="default_string")]
    nvdisasm: String,
    objdumper: Option<String>,
    objdumper_type: Option<String>,
    opt_arg: Option<String>,
    options: Option<String>,
    post_process: Option<Vec<String>>,
    rpath_flag: Option<String>,
    supports_asm_docs: Option<bool>,
    supports_ast_view: Option<bool>,
    supports_binary: Option<bool>,
    supports_cfg: Option<bool>,
    supports_demangle: Option<bool>,
    supports_execute: Option<bool>,
    supports_intel: Option<bool>,
    supports_library_code_filter: Option<bool>,
    supports_opt_output: Option<bool>,
    supports_pp_view: Option<bool>,
    tools: Option<Map<String, Value>>,
    unwise_options: Option<Vec<String>>,
    version: Option<String>,
}

pub fn default_string() -> String {
    "".into()
}

pub fn default_vec_string() -> Vec<String> {
    vec!["".into()]
}
