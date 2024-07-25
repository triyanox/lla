# `lla` - ls, but make it fashion

## what's the deal?
`lla` is like `ls` went to design school. it's fast, it's pretty, it's rust-powered. 
say bye to boring directories. say hi to file listing with style.

<img src="lla.png" alt="lla looking fabulous" width="100%">

## cool tricks
- **speed demon**: blink and you'll miss it
- **long and detailed**: `-l` for when you're feeling nosy
- **sort it out**: `-s` to play favorites (name, size, or date)
- **picky picky**: `-f` to filter files like a bouncer at a club
- **go deeper**: `-r` to dive into subdirectories (how deep? you decide with `-d`)

## get it now
1. **get rust**: no rust? [get rusty](https://www.rust-lang.org/learn/get-started)
2. **magic words**: 
   ```bash
   cargo install lla
   ```
3. **showtime**: type `lla` and watch the magic happen

### fancy netbsd user?
```bash
pkgin install lla
```
(we see you, netbsd. we appreciate you.)

## how to play

```
lla [COOL_FLAGS] [EXTRA_BITS] [WHERE_TO_LOOK]
```

### fun buttons
- `-l`: long list (for the detail-oriented)
- `-r`: recursion (for the adventurous)
- `-g`: git status (for the code-curious)

### extra toppings
- `-s [FLAVOR]`: sort by name, size, or date
- `-f [VIP_LIST]`: filter files (use . for extensions, like a cool kid)
- `-d [HOW_DEEP]`: set max depth (for those afraid of recursion)
- `-h`: help (no shame in asking)
- `-V`: version (for the collectors)

### where to point
- `DIRECTORY`: where to look (or don't, we'll just look here)

## show me the goods

- just the basics:
  ```bash
  lla
  ```
- be specific:
  ```bash
  lla /path/to/your/hopes/and/dreams
  ```
- get all the deets:
  ```bash
  lla -l
  ```
- size matters:
  ```bash
  lla -s size
  ```
- txt files only party:
  ```bash
  lla -f .txt
  ```
- go deep, but not too deep:
  ```bash
  lla -r -d 3
  ```
- kitchen sink special:
  ```bash
  lla -lrs size -f .txt -d 3
  ```

## wanna help?
got ideas? found a bug? think you can make `lla` even cooler? 
we're all ears! pull requests and issues are like fan mail to us.

## boring (but important) stuff
MIT License - do what you want, just don't blame us. 
go forth and `lla` your heart out!
