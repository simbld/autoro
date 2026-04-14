import {Component, inject, OnInit} from "@angular/core";
import {ActivatedRoute} from "@angular/router";
import {TradingService} from "../../services/trading.service";
import {switchMap} from "rxjs";
import {MatCard, MatCardContent} from "@angular/material/card";
import {DecimalPipe} from "@angular/common";
import {MatTab, MatTabGroup} from "@angular/material/tabs";

@Component({
  selector: 'app-markets',
    imports: [
        MatCard,
        MatCardContent,
        DecimalPipe,
        MatTabGroup,
        MatTab
    ],
  templateUrl: './markets.component.html',
  styleUrl: './markets.component.scss'
})
export class MarketsComponent implements OnInit{
    private route = inject(ActivatedRoute);
    private tradingService = inject(TradingService);
    protected symbol = this.route.snapshot.paramMap.get('id');
    public price: number | null = null;
    public variationDay: number | null = null;
    public absDailyPriceChange: number | null = null;
    private widgetLoaded = false
    private loadTradingViewWidget() {
        const script = document.createElement('script');
        script.src = 'https://s3.tradingview.com/tv.js';
        script.onload = () => {
            setTimeout(() => {
                new (window as any).TradingView.widget({
                    container_id: 'tradingview-widget',
                    symbol: 'BINANCE:' + this.symbol + 'USDT',
                    interval: 'D',
                    theme: 'dark',
                    style: '1',
                    locale: 'fr',
                    width: '100%',
                    height: '400',
                });
            }, 100);
        };
        document.head.appendChild(script);
    }

    onTabChange(event: any)  {
        if(event.index === 1 && !this.widgetLoaded) {
            this.widgetLoaded = true;
            this.loadTradingViewWidget();
        }
    }

    ngOnInit() {
        this.load();
    }

    load() {
        if (!this.symbol) return;
        this.tradingService.searchInstrument(this.symbol)
            .pipe(
                switchMap(response => {
                    const item = response.items.find(i => i.internalSymbolFull === this.symbol);
                    this.variationDay = item!.dailyPriceChange;
                    this.absDailyPriceChange = item!.absDailyPriceChange;
                    console.log(item)
                    const id = item!.instrumentId;
                    return this.tradingService.getRates(id.toString());
                })
            )
            .subscribe(d => {
                this.price = d.rates[0].ask;
                this.absDailyPriceChange = this.price * (this.absDailyPriceChange ?? 0) / 100;
            })
    }
}