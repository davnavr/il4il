namespace Il4ilSharp.Interop.Native;

using System.Runtime.InteropServices;

/// <summary>Methods for manipulating IL4IL modules.</summary>
public unsafe static class Module {
    public readonly struct Opaque { }

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_module_create", ExactSpelling = true)]
    public static extern Opaque* Create();

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_module_dispose", ExactSpelling = true)]
    public static extern void Dispose(Opaque* module, out Error.Opaque* error);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_module_validate_and_dispose", ExactSpelling = true)]
    public static extern Error.Opaque* ValidateAndDispose(Opaque* module, out Browser.Opaque* browser);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_module_add_metadata_name", ExactSpelling = true)]
    public static extern void AddMetadataName(Opaque* module, Identifier.Opaque* name);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_module_write_binary_to_path", ExactSpelling = true)]
    public static extern Error.Opaque* WriteBinaryToPath(Opaque* module, Identifier.Opaque* path);

    [DllImport(Library.Name, CallingConvention = CallingConvention.Cdecl, EntryPoint = "il4il_module_write_binary", ExactSpelling = true)]
    public static extern Error.Opaque* WriteBinary(Opaque* module, ByteWriter writer);
}
