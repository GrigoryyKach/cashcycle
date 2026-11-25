# Exchange rates

Cashcycle converts every subscription into the currency of the total card. This page records where the rates come from and why.

## Choice

| Source | Key | UAH | Updates | Terms | Used as |
|---|---|---|---|---|---|
| **ExchangeRate-API, open access** (`https://open.er-api.com/v6/latest/USD`) | none | yes | once per day (the answer carries the time of the next update) | free for personal and commercial use, caching allowed, no re-distribution, **attribution required** ("Rates By Exchange Rate API" with a link to <https://www.exchangerate-api.com>), rate limited | primary |
| **National Bank of Ukraine** (`https://bank.gov.ua/NBUStatService/v1/statdirectory/exchange?json`) | none | yes (official) | daily, on banking days | public open data, no published limits | fallback when the first source fails |
| Frankfurter / ECB | none | **no** | daily | free | rejected: no UAH, and only about 30 currencies |

ExchangeRate-API covers about 160 currencies, UAH included, with a single request, so it is the primary source. The NBU list is smaller (about 45 currencies) but official, so it keeps the common currencies converting when the primary source is down.

Limits as published by ExchangeRate-API for the open endpoint (checked 2026-11-25): a client that requests once every 24 hours never gets limited, and one request per hour is also safe. A limited address gets HTTP 429 for 20 minutes. Cashcycle asks on start, again when its figures are older than 6 hours (checked while it runs), and when the refresh button is pressed; after a failed attempt it waits 15 minutes before trying again by itself.

## How it works

- **One base currency, cross-rates.** The cache stores units of each currency per one unit of the base (USD for ExchangeRate-API, UAH for the NBU). A conversion from A to B is `rate(B) / rate(A)`.
- **Live first, cache as the fallback.** Fresh rates are downloaded on start, every 6 hours while the app runs, and on demand. They are saved to `rates.json` in the data folder with the time they were downloaded. The file is used only when the download fails.
- **Wording.** Normally the card says "Rates updated 25 Nov, 14:02". When the last download failed and cached rates are used it says "No connection · using rates from 25 Nov" (in the warning colour). With no cache at all it says "No exchange rates yet" and the totals are shown per currency; nothing is converted and no number is guessed.
- **A currency without a rate** (for example one from an old or edited data file) is shown unconverted, left out of the converted total, and named in a note on the card.
- **Excluded currencies.** The Russian rouble (RUB) and the Belarusian rouble (BYN) are never offered, searched, defaulted to or converted: they are removed from the currency list and stripped from every rate table as it is read. A subscription that still carries one of them in an old data file is displayed in its own amount with the note above.
- **Arithmetic.** Prices are exact decimals (`rust_decimal`, stored as strings). Only the conversion itself goes through `f64`; results are rounded half to even for display, to 2 decimals (0 for JPY and similar, 3 for KWD and similar). Totals are summed unrounded and rounded once.
- **Networking.** The request is made by the system `curl` (`curl.exe` on Windows), like the update check of the suite, restricted to HTTPS. No cookies, no identifiers; the only thing sent is the request itself.

## Attribution

Required by the open-access terms and shown in two places: Settings, About ("Exchange rates: Exchange Rate API", a link to exchangerate-api.com) and the README.
