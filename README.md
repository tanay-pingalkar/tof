# tof
a function programming language for real world applications made in rust (in development)


## install
- fork the repo
- `git clone`https://github.com/[your_username]/tof.git
- run `cargo install`
- run `cargo run -- example`



## a basic example
!!!! this language is in developement and dont contain too many features.
```
// this is a sum example
sum : _ -> {
  num1 : int (scan "num1 = " )
  num2 : int (scan "num2 = ")
  print "result = " (num1 + num2)
}

sum _
```
