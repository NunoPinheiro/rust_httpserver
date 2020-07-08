# Rust Web Server

Learning rust while implementing a simple http server/framework.

## Todo:
- Support for headers in response
- Support for OPTION method +  CROS configuration
- Use multiple threads
  - Try launching thread per request
  - Try using thread pool workers
  - Try async support
- Implement and test proper keep alive logic
- Implement and test proper shutdown logic
- Benchmark
- Support for static files serving
- Support for template rendering
- Better support for content types + encodings
- Json support/ Check json frameworks
- Rate limit amount of open sockets/in-flight requests
- Expose metrics + Logs
 
## Not planned to suppport:
- TLS termination
