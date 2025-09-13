# Providers and Clients
Below is a list of currently supported providers in `Potato Head`.

## OpenAI

- Chat Completions `v1` API [Documentation](https://platform.openai.com/docs/api-reference/chat)
- Embeddings `v1` API [Documentation](https://platform.openai.com/docs/api-reference/embeddings)

## Google

- Gemini Generate Content API [Documentation](https://ai.google.dev/gemini-api/docs/text-generation)
- Vertex Generate Content `v1beta` API [Documentation](https://cloud.google.com/vertex-ai/generative-ai/docs/reference/rest/v1beta1/projects.locations.endpoints/generateContent)

## Base URLs and Endpoints

| Provider  | Base URL                                                      |  Endpoint (Extended URL)                                  |
|-----------|---------------------------------------------------------------|------------------------------------------------------------------|
| OpenAI    | `https://api.openai.com/v1`                                   | `/chat/completions`, `/embeddings`        |
| Gemini    | `https://generativelanguage.googleapis.com/v1beta/models`     | `/{model}:generateContent`,  `/{model}:embedContent`   |
| Vertex    | `https://{LOCATION}-aiplatform.googleapis.com/{VERTEX_API_VERSION}/projects/{PROJECT_ID}/locations/{LOCATION}/publishers/google/models`          | `/{model}:generateContent`, `/{model}:predict` (for embeddings)                                   |
| Undefined | `https://undefined.provider.url`                               | N/A                                                              |

## Environment Variables
To use the OpenAI or Gemini providers, you need to set the following environment variables:

| Provider  | API Key Environment Variable   | Base Environment Variable      |
|-----------|-------------------------------|------------------------------------|
| OpenAI    | `OPENAI_API_KEY`              | `OPENAI_API_URL` - default: `https://api.openai.com/v1` |
| Gemini    | `GEMINI_API_KEY`              | `GEMINI_API_URL` - default: `https://generativelanguage.googleapis.com/v1beta/models` |
| Vertex    | `GOOGLE_CLOUD_PROJECT`        | None (**Required**)  |
| Vertex    | `GOOGLE_CLOUD_LOCATION`     | us-central1  |
| Vertex    | `VERTEX_API_VERSION`     | v1beta1  |


## Google Authentication

[Reference](https://ai.google.dev/gemini-api/docs/migrate-to-cloud)

As per Google's best practices, both the Gemini and Vertex APIs can be used for LLM applications. However, the Vertex API is designed for production use cases that require specific enterprise controls. As such, the Vertex API authenticates users/services accounts through embedded application credentials instead of a an API key. 

If using the `Vertex` provider, `Potato Head` will attempt to use Google application credentials by default. Please refer to the [Google Application Default Credentials documentation](https://cloud.google.com/vertex-ai/generative-ai/docs/start/gcp-auth) for more information on how to set this up.

**NOTE**: Google application credentials should work for both the `Gemini` and `Vertex` providers.

## Additional Providers
We are currently working on adding support for additional providers, as well as expanding the functionality of existing providers. For instance, in the future `Provider.OpenAI` may evolve into `Provider.OpenAIChat`, `Provider.OpenAIImage`, `Provider.OpenAIResponse`, etc. This will allow for more granular control over the type of response you want to receive from the provider.

If you have a specific provider in mind that you would like to see supported, please let us know, or even better, submit a pull request with the implementation. We welcome contributions from the community to help us expand the capabilities of Potato Head.