# Real-World API Examples for GPT Actions

This document contains examples of OpenAPI specifications for real-world APIs.

## 1. Weather API (Simple GET)
```yaml
openapi: 3.1.0
info:
  title: Weather API
  version: 1.0.0
servers:
  - url: https://api.weather.com/v3
paths:
  /wx/conditions/current:
    get:
      operationId: getCurrentConditions
      parameters:
        - name: location
          in: query
          required: true
          schema:
            type: string
      responses:
        '200':
          description: Current weather conditions
```

## 2. Spotify API (OAuth 2.0)
```yaml
openapi: 3.1.0
info:
  title: Spotify Web API
  version: 1.0.0
servers:
  - url: https://api.spotify.com/v1
paths:
  /me/player/currently-playing:
    get:
      operationId: getCurrentlyPlaying
      responses:
        '200':
          description: Information about the currently playing track
components:
  securitySchemes:
    oauth2:
      type: oauth2
      flows:
        authorizationCode:
          authorizationUrl: https://accounts.spotify.com/authorize
          tokenUrl: https://accounts.spotify.com/api/token
          scopes:
            user-read-currently-playing: Read currently playing track
```
