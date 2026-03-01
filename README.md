# UMass Amherst Web Ring

This is a web ring for UMass Amherst students, alumni, and faculty.
It is a collection of personal websites, blogs, and other web pages.

## How to join
1. Add your website to `members.json` and submit a pull request.
2. Once accepted make sure to add the webring to your website in some way.

### Using our script
> [!Warning]
> If your website is powered by a frontend framework like Svelte, Vue, or React using the script might have strange results. It is suggested to write the integration yourself with the HTTP API indicated below. It's not scary, it's just a `fetch` call.

Add the following script to your website:
```html
<script src="https://umaring.github.io/USERNAME.js"></script>
<div id="umaring"></div>
```

Replace `USERNAME` with your id from `members.json`.

### Building your own integration
Please integrate with the following API:
`GET https://umaring.github.io/USERNAME.json`
This will return a JSON object with the following format:
```json
{
    "prev": {
        "id": "usera",
        "name": "User A",
        "url": "https://usera.com"
    },
    "member": {
        "id": "userb",
        "name": "User B",
        "url": "https://userb.com"
    },
    "next": {
        "id": "userc",
        "name": "User C",
        "url": "https://userc.com"
    }
}
```

## 88x31 Button
If you want to add a button to your website, you can use the following image:
![UMass Amherst Web Ring](umass.png)

You should turn off anti-aliasing for the image to make it look better.
```css
img {
    image-rendering: auto;
    image-rendering: crisp-edges;
    image-rendering: pixelated;
    image-rendering: -webkit-optimize-contrast;
}
```
