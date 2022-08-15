namespace Il4ilSharp;

using System;
using Il4ilSharp.Interop;
using Il4ilSharp.Interop.Native;

/// <summary>Base class for representations of IL4IL module metadata.</summary>
public abstract class ModuleMetadata {
    /// <summary>Gets a handle for the underlying module metadata.</summary>
    public MetadataHandle Handle { get; }

    /// <summary>Gets a value indicating the kind of metadata.</summary>
    public MetadataKind Kind { get; }

    private protected ModuleMetadata(MetadataHandle handle, MetadataKind kind) {
        Handle = handle;
        Kind = kind;
    }

    internal static ModuleMetadata Create(MetadataHandle handle) {
        ArgumentNullException.ThrowIfNull(handle);
        MetadataKind kind = handle.GetKind();
        switch (kind) {
            case MetadataKind.Name:
                return new NameMetadata(handle);
            default:
                throw new InvalidOperationException(kind + " is not a valid metadata kind");
        }
    }
}
