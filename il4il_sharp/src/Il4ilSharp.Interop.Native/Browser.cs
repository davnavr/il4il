namespace Il4ilSharp.Interop.Native;

using System.Runtime.InteropServices;

/// <summary>Methods for manipulating IL4IL module browser instances.</summary>
public unsafe static class Browser {
    public readonly struct Opaque { }

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_browser_dispose", ExactSpelling = true)]
    public static extern void Dispose(Opaque* browser);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_browser_metadata_count", ExactSpelling = true)]
    public static extern nuint MetadataCount(Opaque* browser);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_browser_metadata_copy_to", ExactSpelling = true)]
    public static extern nuint MetadataCopyTo(Opaque* browser, Metadata.Opaque** buffer);
}
