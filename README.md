
<div align="center">
<img src="https://github.com/tanay-pingalkar/tof/blob/main/logo.png"><img>
</div>
<div align="center">
  <h3>tof is a pure functional language made in rust (🚧 in development)</h3>
</div>

## install
- fork the repo
- `git clone https://github.com/[your_username]/tof.git`
- run `cargo install`
- run `cargo run -- example`



## a basic example
```
// this is a sum example
sum : _ -> {
  num1 : int (scan "num1 = " )
  num2 : int (scan "num2 = ")
  print "result = " (num1 + num2)
}

sum _
```

## cli options
run `cargo install` to install tof so you can call tof using `tof` name <br>
- `tof run filename` 
- `tof play` 
- `tof run filename --show-tokens` 
