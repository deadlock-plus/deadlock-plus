# dp-text

Helpers that turn HTML-flavoured feed content into plain text. It is ring 0 because it is pure string code with no dependencies.

## Public API

- `between(text, start, end)` returns the span after the first `start` and before the next `end`, or `None` if either marker is missing.
- `decode_entities(s)` decodes `&nbsp;`, `&lt;`, `&gt;`, `&quot;`, `&amp;` and numeric references such as `&#8203;` and `&#x200B;`.
- `strip_html(html)` removes tags, decodes entities and returns one line per `br`, `p`, `div` or `li`.

## Dependencies

None. No platform-specific code.

## Gotchas

- `decode_entities` leaves a numeric reference alone when it has no digits, no closing `;`, or a value that is not a valid `char`. Steam pads text with `&#8203;`, which is why numeric references are handled and not only named ones.
- `decode_entities` replaces `&amp;` last among the named entities, so `&amp;lt;` decodes to `&lt;` and not `<`.
- `strip_html` also unescapes `\[` to `[` and drops a trailing "Read more" line or suffix. Other sources may lose that text.
- `strip_html` does not parse HTML. It tracks `<` and `>` only, so a bare `<` in text starts a tag that swallows everything up to the next `>`.

## Testing

```
cargo test -p dp-text
```
