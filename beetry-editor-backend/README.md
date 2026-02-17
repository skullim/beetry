## FAQ

1. Why is importing a serialized project/tree not the only prerequisite? Why must the used plugins be loaded?

   Regarding editor: advanced parameter-value validation requires objects that cannot be easily serialized. Serializing all project/tree dependencies would also increase the exported file size and is less flexible than the alternative solution.  
   However, if this is a major concern, one can consider implementing a "limited" editor view that allows only basic operations.

   Regarding tree reconstruction: tree reconstruction requires a constructor that instantiates a given node. This object is not trivially serializable.  
   The chosen approach keeps tree export as small as needed and avoids breaking changes.
