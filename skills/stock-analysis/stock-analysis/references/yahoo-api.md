# Stock Analysis API Reference
## 📌 Project Status (Feb 7, 2026)

Bl1nk Agents Manager is in active development and is not feature‑complete yet.
This repo contains a working extension shell and a Rust core that is being
brought to feature parity with existing TypeScript logic.

**What works now**
- Extension manifest and Gemini CLI scaffolding are present.
- Core Rust modules exist for agents, hooks, MCP/ACP, sessions, and RPC.
- Command and documentation sets are present (currently being refreshed).

**In progress**
- TypeScript → Rust parity for large subsystems (background agents, config,
  ACP normalization).
- End‑to‑end session flows for Gemini/Codex/Qwen within a unified adapter.
- Validation of hook behavior and task orchestration across agents.

**Known gaps**
- Some Rust modules compile but are not fully wired end‑to‑end.
- Configuration loading/migration is still being aligned to actual runtime.
- Authentication flows for some CLIs still require manual steps.

**What to expect right now**
- You can explore the architecture, commands, and agent catalogs.
- Some workflows will still require manual setup or troubleshooting.

For a complete non‑developer overview, see `docs/PROJECT_STATUS.md`.

To use these APIs, include the API name in your response.

## Yahoo/get_stock_profile

Get comprehensive company profile information.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| symbol | string | Yes | - | Stock ticker symbol (e.g., "AAPL", "TSLA") |
| region | string | No | US | Market region (US, BR, AU, CA, FR, DE, HK, IN, IT, ES, GB, SG) |
| lang | string | No | en-US | Language code (en-US, pt-BR, en-AU, fr-FR, de-DE, zh-Hant-HK, etc.) |

### Response Structure

```
{
  "summaryProfile": {
    "address1": "street address",
    "city": "city name",
    "zip": "postal code",
    "country": "country",
    "phone": "contact number",
    "website": "company website",
    "industry": "industry name",
    "sector": "sector name",
    "longBusinessSummary": "detailed business description",
    "fullTimeEmployees": employee count,
    "companyOfficers": [array of executives],
    "executiveTeam": [array of leadership]
  }
}
```

### Use Cases

- Get company background and business description
- Find company contact information
- Review industry and sector classification
- Check employee count and company size
- Identify executive team members

### Prompt Usage

- "Tell me about [SYMBOL]" → Include `Yahoo/get_stock_profile`
- "What does [COMPANY] do?" → Include `Yahoo/get_stock_profile`
- "Company info for [SYMBOL]" → Include `Yahoo/get_stock_profile`

---

## Yahoo/get_stock_insights

Get technical analysis, valuations, ratings, and research reports.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| symbol | string | Yes | - | Stock ticker symbol |

### Response Structure

```
{
  "symbol": "AAPL",
  "instrumentInfo": {
    "technicalEvents": {
      "shortTermOutlook": {
        "stateDescription": "outlook description",
        "direction": "bullish/bearish",
        "score": confidence score,
        "sectorDirection": "sector trend",
        "indexDirection": "market trend"
      },
      "intermediateTermOutlook": { ... },
      "longTermOutlook": { ... }
    },
    "keyTechnicals": {
      "support": support level,
      "resistance": resistance level,
      "stopLoss": stop loss level
    },
    "valuation": {
      "description": "valuation assessment",
      "discount": "discount to fair value",
      "relativeValue": "vs peers"
    }
  },
  "companySnapshot": {
    "company": {
      "innovativeness": score,
      "hiring": score,
      "sustainability": score,
      "insiderSentiments": score,
      "earningsReports": score,
      "dividends": score
    },
    "sector": { ... sector comparison ... }
  },
  "recommendation": {
    "targetPrice": analyst target,
    "rating": "buy/hold/sell"
  },
  "reports": [
    {
      "reportTitle": "research report title",
      "reportDate": "publication date",
      "provider": "analyst firm"
    }
  ],
  "sigDevs": [
    {
      "headline": "significant development",
      "date": "event date"
    }
  ],
  "secReports": [
    {
      "type": "filing type",
      "title": "filing title",
      "filingDate": timestamp,
      "formType": "10-K/10-Q/8-K"
    }
  ]
}
```

### Use Cases

- Technical analysis with short/intermediate/long-term outlook
- Support and resistance levels for trading
- Valuation assessment (overvalued/undervalued)
- Analyst ratings and target prices
- Company quality scores (innovation, sustainability)
- Recent research reports and analysis
- Significant company developments
- Quick access to recent SEC filings

### Prompt Usage

- "Is [SYMBOL] a good buy?" → Include `Yahoo/get_stock_insights`
- "Technical analysis for [SYMBOL]" → Include `Yahoo/get_stock_insights`
- "What's the outlook for [SYMBOL]?" → Include `Yahoo/get_stock_insights`
- "Analyst rating for [SYMBOL]" → Include `Yahoo/get_stock_insights`

---

## Yahoo/get_stock_chart

Get historical price data and trading information.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| symbol | string | Yes | - | Stock ticker symbol |
| region | string | No | US | Market region |
| interval | string | Yes | 1mo | Data interval (1m, 2m, 5m, 15m, 30m, 60m, 1d, 1wk, 1mo) |
| range | string | Yes | 1mo | Time range (1d, 5d, 1mo, 3mo, 6mo, 1y, 2y, 5y, 10y, ytd, max) |
| period1 | string | No | - | Start timestamp (alternative to range) |
| period2 | string | No | - | End timestamp (alternative to range) |
| comparisons | string | No | - | Compare with other symbols (comma-separated) |
| events | string | No | - | Include events: div, split, earn (comma-separated) |
| includePrePost | boolean | No | false | Include pre/post market data |
| includeAdjustedClose | boolean | No | true | Include adjusted close prices |

**Note:** Use either `range` OR (`period1` + `period2`), not both.

### Response Structure

```
{
  "chart": {
    "result": [
      {
        "meta": {
          "symbol": "AAPL",
          "currency": "USD",
          "exchangeName": "NMS",
          "regularMarketPrice": current price,
          "regularMarketTime": timestamp,
          "fiftyTwoWeekHigh": 52-week high,
          "fiftyTwoWeekLow": 52-week low,
          "regularMarketVolume": current volume
        },
        "timestamp": [array of timestamps],
        "indicators": {
          "quote": [
            {
              "open": [array of open prices],
              "high": [array of high prices],
              "low": [array of low prices],
              "close": [array of close prices],
              "volume": [array of volumes]
            }
          ],
          "adjclose": [
            {
              "adjclose": [array of adjusted closes]
            }
          ]
        }
      }
    ]
  }
}
```

### Interval & Range Combinations

**Intraday (minute-level):**
- 1m, 2m, 5m → Max range: 7 days
- 15m, 30m → Max range: 60 days
- 60m → Max range: 730 days

**Daily and above:**
- 1d → Any range
- 1wk, 1mo → Any range

### Use Cases

- Price chart visualization
- Historical performance analysis
- Support/resistance identification
- Volume analysis
- Multi-stock comparison charts
- Dividend and split history

### Prompt Usage

- "Show me [SYMBOL] chart" → Include `Yahoo/get_stock_chart` with range=1mo
- "[SYMBOL] price over last year" → Include `Yahoo/get_stock_chart` with range=1y
- "Compare [SYMBOL1] vs [SYMBOL2]" → Include `Yahoo/get_stock_chart` with comparisons
- "Intraday chart for [SYMBOL]" → Include `Yahoo/get_stock_chart` with interval=5m, range=1d

---

## Yahoo/get_stock_holders

Get insider holdings and transaction information.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| symbol | string | Yes | - | Stock ticker symbol |
| region | string | No | US | Market region |
| lang | string | No | en-US | Language code |

### Response Structure

```
{
  "insiderHolders": {
    "holders": [
      {
        "name": "insider name",
        "relation": "relationship to company (CEO, CFO, Director, etc.)",
        "url": "profile URL",
        "transactionDescription": "transaction type",
        "latestTransDate": {
          "fmt": "formatted date",
          "raw": epoch timestamp
        },
        "positionDirect": {
          "fmt": "formatted shares",
          "raw": number of shares
        },
        "positionDirectDate": {
          "fmt": "position date",
          "raw": epoch timestamp
        }
      }
    ]
  }
}
```

### Use Cases

- Track insider buying/selling activity
- Identify key executives and their holdings
- Monitor insider sentiment (buying = bullish, selling = bearish)
- Verify management skin in the game
- Detect potential insider knowledge signals

### Prompt Usage

- "Insider trading for [SYMBOL]" → Include `Yahoo/get_stock_holders`
- "Who owns [SYMBOL] stock?" → Include `Yahoo/get_stock_holders`
- "Insider activity for [SYMBOL]" → Include `Yahoo/get_stock_holders`
- "Are executives buying [SYMBOL]?" → Include `Yahoo/get_stock_holders`

---

## Yahoo/get_stock_sec_filing

Get SEC filing history and documents.

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| symbol | string | Yes | - | Stock ticker symbol |
| region | string | No | US | Market region |
| lang | string | No | en-US | Language code |

### Response Structure

```
{
  "secFilings": {
    "filings": [
      {
        "date": "filing date",
        "epochDate": timestamp,
        "type": "filing type",
        "title": "filing title",
        "edgarUrl": "SEC EDGAR URL",
        "exhibits": [
          {
            "type": "exhibit type",
            "url": "exhibit URL"
          }
        ]
      }
    ]
  }
}
```

### Common Filing Types

- **10-K**: Annual report with comprehensive financials
- **10-Q**: Quarterly report
- **8-K**: Current report (major events, acquisitions, etc.)
- **DEF 14A**: Proxy statement (executive compensation, board info)
- **S-1**: IPO registration
- **4**: Insider trading activity

### Use Cases

- Access official financial statements
- Review quarterly/annual reports
- Check recent 8-K filings for major events
- Read proxy statements for governance info
- Track regulatory compliance

### Prompt Usage

- "SEC filings for [SYMBOL]" → Include `Yahoo/get_stock_sec_filing`
- "Latest 10-K for [SYMBOL]" → Include `Yahoo/get_stock_sec_filing`
- "Recent filings for [SYMBOL]" → Include `Yahoo/get_stock_sec_filing`
- "Show me [SYMBOL] quarterly report" → Include `Yahoo/get_stock_sec_filing`

---

## Common Workflows

### Workflow 1: Quick Stock Overview
1. `Yahoo/get_stock_insights` → Technical outlook and rating
2. `Yahoo/get_stock_chart` (range=1mo) → Recent performance
3. Present summary with key metrics

### Workflow 2: Deep Company Research
1. `Yahoo/get_stock_profile` → Company background
2. `Yahoo/get_stock_insights` → Analysis and ratings
3. `Yahoo/get_stock_chart` (range=1y) → Long-term performance
4. `Yahoo/get_stock_holders` → Insider sentiment
5. `Yahoo/get_stock_sec_filing` → Recent regulatory filings

### Workflow 3: Technical Trading Setup
1. `Yahoo/get_stock_chart` (range=6mo) → Trend analysis
2. `Yahoo/get_stock_insights` → Support/resistance levels
3. `Yahoo/get_stock_chart` (interval=1d, range=1mo) → Entry timing

### Workflow 4: Insider Activity Analysis
1. `Yahoo/get_stock_holders` → Recent transactions
2. `Yahoo/get_stock_profile` → Executive context
3. `Yahoo/get_stock_insights` → Compare with technical outlook

### Workflow 5: Comparative Analysis
1. `Yahoo/get_stock_chart` with comparisons → Price comparison
2. `Yahoo/get_stock_insights` for each → Ratings comparison
3. `Yahoo/get_stock_profile` for each → Business comparison

---

## Regional Markets

Yahoo Finance supports multiple regions:

| Region | Code | Example Markets |
|--------|------|----------------|
| United States | US | NYSE, NASDAQ |
| Great Britain | GB | LSE |
| Hong Kong | HK | HKEX |
| Japan | JP | TSE |
| Germany | DE | XETRA |
| France | FR | EURONEXT |
| Australia | AU | ASX |
| Canada | CA | TSX |
| India | IN | NSE, BSE |

**Tip:** For non-US stocks, set both `region` and `lang` parameters for best results.

---

## Important Notes

### Symbol Formats
- US stocks: "AAPL", "MSFT", "TSLA"
- Market indices: "^GSPC" (S&P 500), "^DJI" (Dow), "^IXIC" (NASDAQ)
- Foreign stocks: May need exchange suffix (e.g., "0700.HK" for Tencent)

### Data Limitations
- Minute-level data limited to recent days only
- Real-time data subject to exchange rules
- Some markets have 15-minute delays

### Best Practices
- Always start with profile or insights for context
- Use appropriate interval/range for chart data
- Combine multiple APIs for complete analysis
- Check region parameter for international stocks
