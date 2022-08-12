namespace Il4ilSharp.Interop.Native;

using System.Runtime.InteropServices;

/// <summary>Methods for manipulating IL4IL identifier strings.</summary>
public unsafe static class Identifier
{
    public readonly struct Opaque { }

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_identifier_from_utf8", ExactSpelling = true)]
    public static extern Opaque* FromUtf8(byte* contents, nuint length, out Error.Opaque* error);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_identifier_contents", ExactSpelling = true)]
    public static extern byte* Contents(Opaque* identifier, out nuint length, out Error.Opaque* error);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_identifier_dispose", ExactSpelling = true)]
    public static extern void Dispose(Opaque* identifier, out Error.Opaque* error);
}
