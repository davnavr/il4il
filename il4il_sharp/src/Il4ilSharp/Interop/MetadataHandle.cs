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
}
