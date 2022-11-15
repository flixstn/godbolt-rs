#[cfg(test)]
mod tests {
    use crate::*;

    #[tokio::test]
    async fn test_get_compilers() -> Result<(), Box<dyn Error>> {
        let godbolt_api = Godbolt::new();
        let compilers = godbolt_api.get_compilers().await;
        assert!(compilers.is_ok());
        
        Ok(())
    }

    #[tokio::test]
    async fn test_get_compilers_with_all_fields() -> Result<(), Box<dyn Error>> {
        let godbolt_api = Godbolt::new();
        let compilers = godbolt_api.get_compilers_with_all_fields().await;
        assert!(compilers.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_getcompiler_with_fields() -> Result<(), Box<dyn Error>> {
        let godbolt_api = Godbolt::new();
        let compilers = godbolt_api.get_compilers_with_fields(&vec!["supportsAsmDocs", "version", "tools"]).await;
        assert!(compilers.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_library_by_id() -> Result<(), Box<dyn Error>> {
        let godbolt_api = Godbolt::new();
        let library = godbolt_api.get_library_by_id("rust").await;
        assert!(library.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_get_formats() -> Result<(), Box<dyn Error>> {
        let godbolt_api = Godbolt::new();
        let formatters = godbolt_api.get_formats().await;
        assert!(formatters.is_ok());

        Ok(())
    }
 
    #[tokio::test]
    async fn test_send_request() -> Result<(), Box<dyn Error>> {
        let godbolt_api = Godbolt::new();
        let src = r#"
            pub fn square(num: i32) -> i32 {
                num * num
            }
        "#;

        let compilation_response = godbolt_api.send_request("r1600", src).await;
        assert!(compilation_response.is_ok());
        Ok(())
    }
    #[tokio::test]
    async fn test_send_request_with_options() -> Result<(), Box<dyn Error>> {
        let godbolt_api = Godbolt::new();
        let src = r#"
            pub fn square(num: i32) -> i32 {
                num * num
            }
        "#;
        let filters = Filters::default();
        let options = Options{user_arguments: "-O".into(), filters};
        let compilers = godbolt_api.send_request_with_options("r1600", src, options).await;
        
        assert!(compilers.is_ok());
        Ok(())
    }
}