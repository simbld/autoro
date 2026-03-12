#!/bin/bash
curl -s \
  "https://public-api.etoro.com/api/v1/trading/info/demo/pnl" \
  -H "x-api-key: sdgdskldFPLGfjHn1421dgnlxdGTbngdflg6290bRjslfihsjhSDsdgGHH25hjf" \
  -H "x-user-key: eyJjaSI6IjYwY2FiYjBiLTU1OTctNDQ4NS04ZjYzLTdlOWUwNTZlMGJiOCIsImVhbiI6IlVucmVnaXN0ZXJlZEFwcGxpY2F0aW9uIiwiZWsiOiItemFUYnYzSlM4V08tcGJhRmx2c29kby1OOGVwUXJxOXI0MGFuVlJIZFVta1FFLU5GbHRucS43Qmk5TGp1WnlFSVlmS0dDVS5CZVprZEwuTms1QlJXS2llOFJMb21NclpnZ0JOU3k0S0htY18ifQ__" \
  -H "x-request-id: $(cat /proc/sys/kernel/random/uuid)" | head -c 500
