namespace Il4ilSharp.Interop.Native;

using System.Runtime.InteropServices;

/// <summary>Methods for manipulating IL4IL module metadata.</summary>
public unsafe static class Metadata {
    public struct Opaque { }

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_metadata_kind", ExactSpelling = true)]
    public static extern MetadataKind Kind(Opaque* metadata);
}
