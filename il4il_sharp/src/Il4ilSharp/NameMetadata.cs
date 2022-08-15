namespace Il4ilSharp;

using System;
using Il4ilSharp.Interop;

/// <summary>Indicates the name of an IL4IL module in a metadata section.</summary>
public sealed class NameMetadata : ModuleMetadata {
    internal NameMetadata(MetadataHandle handle) : base(handle, Interop.Native.MetadataKind.Name) {
        
    }
}
