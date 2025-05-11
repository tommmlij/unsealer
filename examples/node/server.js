const express = require('express');
const app = express();

app.get('/', (req, res) => {
    const secret = process.env.SECRET;
    res.send(`<!DOCTYPE html><html><head><title>Hello</title></head><body><div>Hello ${secret}!</div></body></html>`);
});

app.listen(3000, () => {
    console.log('Server running at http://localhost:3000');
});
