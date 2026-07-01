import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
  name: 'instrumentName',
    standalone: true
})
export class InstrumentNamePipe implements PipeTransform {
    private readonly mapping: { [key: number]: string } = {
        100063: 'SOL',
        100001: 'ETH',
        100000: 'BTC'
    }

    transform(value: number): string {
    return this.mapping[value] ?? 'Unknown';
  }

}
