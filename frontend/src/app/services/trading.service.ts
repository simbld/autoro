import {inject, Injectable} from "@angular/core";
import {HttpClient} from "@angular/common/http";
import {Observable} from "rxjs";

export interface Position {
    positionID: number;
    instrumentID: number;
    isBuy: boolean;
    leverage: number;
    units: number;
    amount: number;
    openRate: number;
    stopLossRate: number | null;
    takeProfitRate: number | null;
    isTslEnabled: boolean;
    tslRate: number | null;
}

export interface Portfolio {
    credit: number;
    positions: Position[];
}

export interface InstrumentRate {
    instrumentID: number;
    ask: number;
    bid: number;
    lastExecution: number | null;
    date: string | null;
    rates: number[];
}

export interface InstrumentRatesResponse {
    rates: InstrumentRate [];
}

export interface InstrumentSearchItem {
    instrumentId: number;
    internalSymbolFull: string;
    instrumentDisplayName: string | null;
    dailyPriceChange: number | null;
    absDailyPriceChange: number | null;
}

export interface InstrumentSearchResponse {
    items: InstrumentSearchItem [];
}

export interface NewsArticle {
    title: string;
    description: string;
    url: string;
    imageUrl: string;
    publishedAt: number;
    sourceName: string;
}

@Injectable({
    providedIn: "root"
})
export class TradingService {
    private http = inject(HttpClient);
    private api = "http://localhost:8081/api";

    getPortfolio(): Observable<Portfolio> {
        return this.http.get<Portfolio>(`${this.api}/portfolio`);
    }

    closePosition(positionId: number, instrumentId: number): Observable<unknown> {
        return this.http.post(`${this.api}/positions/${positionId}/close?instrumentId=${instrumentId}`, {});
    }

    getRates(ids: string): Observable<InstrumentRatesResponse> {
        return this.http.get<InstrumentRatesResponse>(`${this.api}/instruments/rates?ids=${ids}`);
    }

    searchInstrument(symbol: string): Observable<InstrumentSearchResponse> {
        return  this.http.get<InstrumentSearchResponse>(`${this.api}/instruments/search?symbol=${symbol}`);
    }

    getNews(symbol: string): Observable<{articles: NewsArticle[]}> {
        return  this.http.get<{articles: NewsArticle[]}>(`${this.api}/instruments/news?symbol=${symbol}`);
    }
}
