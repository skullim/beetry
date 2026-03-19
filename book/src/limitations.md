# Limitations

This chapter collects current framework constraints and known limitations.

1. A node may define at most one sender port and at most one receiver port for a
   given message type.

   Possible workarounds:

   - Wrap each message type in a distinct newtype.
     This avoids the type collision, but still requires leaf nodes that convert
     between the original type and the wrapper type.
   - Pack multiple fields into a more compound struct.
     However, publishing the more compound type may be more difficult.
  