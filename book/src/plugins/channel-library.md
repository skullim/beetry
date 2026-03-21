# Channel Library

The built-in channel implementations live in the `beetry-channel` crate. It
contains all channel kinds currently supported by Beetry and provides the
delivery behaviors that can be exposed through channel plugins.

If your use case needs a different delivery model, contributions are welcome.
Opening a pull request for a new channel kind is encouraged.

>[!NOTE]
> At the moment, adding a new channel kind requires changes in several places,
> so it is best to get in touch first to coordinate the work.

Beetry currently provides the following channel kinds:

- `mpsc`
- `watch`
- `broadcast`

## Selecting a channel

- use `mpsc` when messages should be processed one by one and every queued item
  matters
- use `watch` when only the latest value matters, such as status or sensor
  state
- use `broadcast` when the same message should be delivered to multiple
  receivers

>[!IMPORTANT]
> With `mpsc`, keep in mind that once a sender successfully sends a message,
> aborting any intermediate node on the execution path might not clear the
> receiver buffer. In the worst case, stale messages can fill the bounded buffer
> and cause later sends to fail.
> To minimize this risk, the sender and receiver should ideally be neighboring
> nodes in the execution path.
