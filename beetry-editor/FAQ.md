1. Why importing serialized project/tree is not the only prerequisite, i.e. why must plugins be available? 
Regarding editor: providing advanced parameter value validation requires objects that cannot be easily serialized. Serializing all project dependencies also increases the size of the exported file and is less
flexible than alternative version.
However, if that is a major point one can consider implementing "limited" editor view that allows to perform basic operations.

Regarding tree reconstruction: For tree reconstruction one needs constructor that instantiates given node.
This object is not trivially serializable. This approach allows to tree export to be as small as possible and does not cause breaking changes. 
