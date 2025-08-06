# Providers and Clients
Below is a list of currently supported providers in `Potato Head`.

## OpenAI
The OpenAI provider type currently supports the ChatCompletion API.

## Gemini
The Gemini provider type currently supports the generativelanguage models API.

## Base URLs and Endpoints

| Provider  | Base URL                                                      |  Endpoint (Extended URL)                                  |
|-----------|---------------------------------------------------------------|------------------------------------------------------------------|
| OpenAI    | `https://api.openai.com/v1`                                   | `/chat/completions`                                              |
| Gemini    | `https://generativelanguage.googleapis.com/v1beta/models`     | `/{model}:generateContent`                                       |
| Undefined | `https://undefined.provider.url`                               | N/A                                                              |

## Environment Variables
To use the OpenAI or Gemini providers, you need to set the following environment variables:

| Provider  | API Key Environment Variable   | Base URL Environment Variable      |
|-----------|-------------------------------|------------------------------------|
| OpenAI    | `OPENAI_API_KEY`              | `OPENAI_API_URL` - default: `https://api.openai.com/v1` |
| Gemini    | `GEMINI_API_KEY`              | `GEMINI_API_URL` - default: `https://generativelanguage.googleapis.com/v1beta/models` |


## Additional Providers
We are currently working on adding support for additional providers, as well as expanding the functionality of existing providers. For instance, in the future `Provider.OpenAI` may evolve into `Provider.OpenAIChat`, `Provider.OpenAIImage`, `Provider.OpenAIResponse`, etc. This will allow for more granular control over the type of response you want to receive from the provider.

If you have a specific provider in mind that you would like to see supported, please let us know, or even better, submit a pull request with the implementation. We welcome contributions from the community to help us expand the capabilities of Potato Head.