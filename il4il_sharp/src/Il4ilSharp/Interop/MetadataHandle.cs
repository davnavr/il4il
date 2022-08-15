namespace Il4ilSharp.Interop;

using System;
using Il4ilSharp.Interop.Native;

/// <summary>Provides a thread-safe wrapper over a reference to IL4IL module metadata.</summary>
public unsafe sealed class MetadataHandle : DerivedHandle<BrowserHandle, Browser.Opaque, Metadata.Opaque> {
    internal MetadataHandle(BrowserHandle browser, Metadata.Opaque* metadata) : base(browser, metadata) { }

    /// <summary>Gets the kind of metadata.</summary>
    public MetadataKind GetKind() {
        try {
            var metadata = Lock();
            return Metadata.Kind(metadata);
        } finally {
            Unlock();
        }
    }

    /// <summary>Attempts to retrieve a module name.</summary>
    public IdentifierHandle? GetModuleName() {
        try {
            var metadata = Lock();
            Identifier.Opaque* name = Metadata.ModuleName(metadata);
            if (name == null) {
                return null;
            }

            return new IdentifierHandle(name);
        } finally {
            Unlock();
        }
    }
}
