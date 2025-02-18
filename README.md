Shows calendar events on terminal using `.ics` file format.
I use this for birthdays. Shows all birthdays on current day and next 7 days.

```text
# command: bdays
# Sample output
-----
TODAY
-----
> AOE's Birthday

--------
TOMORROW
--------
> Definitive Edition's Birthday

--------
UPCOMING
--------
 1 day  | Sat 22/02 | Dave's Birthday
 2 days | Sat 23/02 | T90 official's Birthday
 2 days | Sat 23/02 | Memb's Birthday
 3 days | Sat 24/02 | Daniela's Birthday
 6 days | Sat 27/02 | Lidakor's Birthday

```

## Usage:
- need to have a calendar `cal.ics` file in your $HOME directory. I have only tested calendar file exported from outlook
- need to have rust with cargo installed
- run `cargo build --release` to produce binary and copy to path
- run `bdays` in a terminal


