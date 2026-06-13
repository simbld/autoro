import { Pipe, PipeTransform } from '@angular/core';

@Pipe({
  name: 'instrumentName',
    standalone: true
})
export class InstrumentNamePipe implements PipeTransform {
    private readonly mapping: { [key: number]: string } = {
        100000: 'SOL',
        100001: 'BTC',
        100063: 'ETH'
    }

    transform(value: number): string {
    return this.mapping[value];
  }

}
