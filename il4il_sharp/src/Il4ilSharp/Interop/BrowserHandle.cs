namespace Il4ilSharp.Interop;

using System;
using Il4ilSharp.Interop.Native;

/// <summary>
/// Provides a thread-safe wrapper for an IL4IL module browser.
/// </summary>
/// <seealso cref="ModuleHandle.ValidateAndDispose"/>
public unsafe sealed class BrowserHandle : SharedHandle<Browser.Opaque> {
    internal BrowserHandle(Browser.Opaque* browser) : base(browser) { }

    /// <summary>
    /// Creates an array containing <see cref="MetadataHandle"/> instances representing the contents of the module's metadata sections.
    /// </summary>
    public MetadataHandle[] GetMetadata() {
        try {
            Browser.Opaque* browser = Lock();
            Metadata.Opaque*[] references = new Metadata.Opaque*[Browser.MetadataCount(browser)];

            fixed (Metadata.Opaque** buffer = references) {
                Browser.MetadataCopyTo(browser, buffer);
            }

            MetadataHandle[] metadata = new MetadataHandle[references.Length];

            for (int i = 0; i < metadata.Length; i++) {
                metadata[i] = new MetadataHandle(this, references[i]);
            }

            return metadata;
        } finally {
            Unlock();
        }
    }

    private protected override unsafe void Cleanup(Browser.Opaque* pointer) {
        Browser.Dispose(pointer);
    }
}
