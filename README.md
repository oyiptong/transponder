# transponder

Transponder is a utility that receives payloads that have to be non-blocking and forwards them to endpoints with less strict requirements

The supported means are JSON over UDP -> HTTP

# guarantees

* Messages are sent approximately in the order they are received, but there are no guarantees
* There is natural queuing of upload jobs, but it is not-persistent

# implementation

Input is evented, output is managed through a threadpool.

Using the threadpool is a good way to control concurrent access to the API endpoint.
