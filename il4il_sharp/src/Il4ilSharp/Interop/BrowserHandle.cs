namespace Il4ilSharp.Interop;

using System;
using Il4ilSharp.Interop.Native;

/// <summary>
/// Provides a thread-safe wrapper for an IL4IL module browser.
/// </summary>
/// <seealso cref="ModuleHandle.ValidateAndDispose"/>
public unsafe sealed class BrowserHandle : SharedHandle<Browser.Opaque> {
    internal BrowserHandle(Browser.Opaque* browser) : base(browser) { }

    public MetadataReference[] GetMetadata() {
        try {
            Browser.Opaque* browser = Lock();
            var metadata = new MetadataReference[Browser.MetadataCount(browser)];
            Metadata.Opaque** references = stackalloc Metadata.Opaque*[metadata.Length];

            throw new System.NotImplementedException();
            return metadata;
        } finally {
            Unlock();
        }
    }

    private protected override unsafe void Cleanup(Browser.Opaque* pointer) {
        Browser.Dispose(pointer);
    }
}
