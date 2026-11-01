import fastapi_poe as fp
from typing import AsyncIterable

class CustomBot(fp.PoeBot):
    async def get_response(
        self, request: fp.QueryRequest
    ) -> AsyncIterable[fp.PartialResponse]:
        # Implementation for {bot_name}
        # Bot Type: {bot_type}
        # Base Model: {base_model}
        
        async for msg in fp.stream_request(
            request, "{base_model}", request.access_key
        ):
            yield msg

if __name__ == "__main__":
    fp.run(CustomBot(), allow_without_key=True)
