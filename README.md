# stream-rate-limiter

Stream combinator

.rate_limiter(...)

Provides way to limit stream element rate with constant intervals. 
It adds some level of customization in case of stream delays 
inaccessible in "standard" solution (tokio-timer)

Before you use this library, please consider using tokio-timer crate
(you can look at tokio_interval example)

If you need to limit stream rate with constant intervals, with additional
customisation (increasing permanent stream delay in case of stream delays, which
prevents flooding stream after huge hickups.

For example you want to produce a stream of elements with constant rate of 1 element per second.
You can use this library to set rate limit to 1 element and for example 1 second of accepted delay.
Thus after your stream is stuck for like couple of seconds your event won't be flooding but they go back to rate at one 
per second fast. You can also react and close the stream if delay is too big.



![alt text](https://github.com/[username]/[reponame]/blob/[branch]/image.jpg?raw=true)



