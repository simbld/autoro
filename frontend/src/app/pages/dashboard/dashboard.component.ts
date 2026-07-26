import {Component, DestroyRef, inject} from "@angular/core";
import {takeUntilDestroyed} from "@angular/core/rxjs-interop";
import {CommonModule} from "@angular/common";
import {MatTableModule} from "@angular/material/table";
import {MatButtonModule} from "@angular/material/button";
import {MatCardModule} from "@angular/material/card";
import {Portfolio, Position, TradingService} from "../../services/trading.service";
import {interval, merge, of, Subject} from "rxjs";
import {map, startWith, switchMap, tap} from "rxjs/operators";
import {InstrumentNamePipe} from "../../pipes/instrument-name.pipe";

@Component({
  selector: 'app-dashboard',
    imports: [CommonModule, MatTableModule, MatButtonModule, MatCardModule, InstrumentNamePipe],
  templateUrl: './dashboard.component.html',
  styleUrl: './dashboard.component.scss'
})

export class DashboardComponent {
    private trading = inject(TradingService);
    private destroyRef = inject(DestroyRef);
    private refresh$ = new Subject<void>();
    portfolio: Portfolio | null = null;
    prices: Record<number, number> = {}
    columns = ['instrument', 'direction', 'openRate', 'amount', 'current', 'sl', 'tp', 'tsl', 'close'];

    constructor() {
        merge(interval(30000), this.refresh$).pipe(
            startWith(0),
            switchMap(() => this.trading.getPortfolio()),
            tap(p => this.portfolio = p),
            switchMap((p: Portfolio) => {
                const ids = [...new Set(p.positions.map((x: Position) => x.instrumentID))];
                return ids.length ? this.trading.getRates(ids.join(',')) : of({rates: []});
            }),
            map(r => Object.fromEntries(
                r.rates.map(x => [x.instrumentID, (x.ask + x.bid) / 2])
            ) as Record<number, number>),
            takeUntilDestroyed(this.destroyRef)
        ).subscribe(prices => this.prices = prices);
    }

    close(p: { positionID: number, instrumentID: number }) {
        this.trading.closePosition(p.positionID, p.instrumentID)
            .pipe(takeUntilDestroyed(this.destroyRef))
            .subscribe(() => this.refresh$.next());
    }
}
