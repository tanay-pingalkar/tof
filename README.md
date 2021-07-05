# tof
a function programming language for real world applications made in rust (in development)


## install
- fork the repo
- `git clone`https://github.com/[your_username]/tof.git
- run `cargo install`
- run `cargo run -- example`


## a basic example
!!!! this is language is in developement and dont contain too many features.
```
// this is a sum example
sum : () -> {
  num1 : int(stdin("num1 = "))
  num2 : int(stdin("num2 = "))
  stdout("result = ", num1 + num2)
}

sum()
```
