<h1 align="center">
  <img src="docs/logo.png" alt="frogment" width="200">
  <br>frogment<br>
</h1>

a DNS resolver that fragments domain names using [RFC 1035](https://datatracker.ietf.org/doc/html/rfc1035#section-4.1.4) compression pointers to bypass gfw dns poisoning.

## Usage
```sh
$ frogment --config ./example-config.toml
```

it's mostly useful alongside tools like [dnstt](https://www.bamsoftware.com/software/dnstt/).

E.g. if your domain is blocked by the gfw, you can use frogment to bypass domain-level censorship and dnstt to tunnel your traffic on dns using your domain.

```sh
$ frogment --config ./example-config.toml &
$ ./dnstt -udp 127.0.0.1:1053 -pubkey YOUR_PUBKEY YOUR_FILTERED_DOMAIN 127.0.0.1:1088
```

## How it works
normally a domain is encoded as a sequence of labels:
```
[length][label bytes]...[0]
```
E.g. www.example.com becomes:
```
03 'w' 'w' 'w' 07 'e' 'x' 'a' 'm' 'p' 'l' 'e' 03 'c' 'o' 'm' 00
```
to reduce packet size, [rfc1035](https://datatracker.ietf.org/doc/html/rfc1035#section-4)
allows a qname to be replaced with a pointer (or chain of pointers).
`frogment` manipulates label pointers to shift the qname offset inside the packets.
```
[pointer]...[pointer][label bytes]
```
it forces DPI systems like gfw to follow offsets, which makes name reconstruction costly and harder.

## Demo
![demo](docs/demo.png)
