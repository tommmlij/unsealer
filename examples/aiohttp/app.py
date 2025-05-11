import os

from aiohttp import web

async def hello(request):
    return web.Response(text=f"Hello {os.environ.get('SECRET', 'my missing secret')}!")

app = web.Application()
app.router.add_get('/', hello)

if __name__ == '__main__':
    web.run_app(app, port=3000)

 