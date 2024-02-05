## Rando rust twitch chat-primarily bot to tinker with few things

It can:
- Keep a track of individual live stream viewer's watch streak
- Spy on the chat and grab incoming messages for nefarious purposes

To start you need twitch bot (app) id+secret and generate in whatever way a sample token, store that in .env as `BOT_TOKEN`, and specify as which user to log in (the one which got the token) as `BOT_USERNAME`. To start stalking channels and store data start an redis instance and fill `channels` list with twitch channel logins (lowercase, yada-yada).
