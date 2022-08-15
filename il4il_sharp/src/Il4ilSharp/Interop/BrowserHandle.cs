namespace Il4ilSharp.Interop;

using System;
using Il4ilSharp.Interop.Native;

/// <summary>
/// Provides a thread-safe wrapper for an IL4IL module browser.
/// </summary>
/// <seealso cref="ModuleHandle.ValidateAndDispose"/>
public unsafe sealed class BrowserHandle : SyncHandle<Browser.Opaque> {
    // TODO: If having a Dispose method is bad design, then make a class Inner : SyncHandle
    // For now, objects that contain pointers derived from a browser pointer will simply keep a reference to a BrowserHandle, and
    // periodically check if the BrowserHandle has been disposed.

    internal BrowserHandle(Browser.Opaque* browser) : base(browser) { }

    public MetadataReference[] GetMetadata() {
        try {
            Browser.Opaque* browser = Enter();
            var metadata = new MetadataReference[Browser.MetadataCount(browser)];
            Metadata.Opaque** references = stackalloc Metadata.Opaque*[metadata.Length];

            throw new System.NotImplementedException();
            return metadata;
        } finally {
            Exit();
        }
    }

    private protected override unsafe void Cleanup(Browser.Opaque* pointer) {
        Browser.Dispose(pointer);
    }
}
