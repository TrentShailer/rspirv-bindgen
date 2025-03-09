[vk::push_constant]
ConstantBuffer<uint> push_uint;

[Shader("compute")]
[NumThreads(1, 1, 1)]
bool compute_main()
{
    return push_uint == 1;
}

[Shader("vertex")]
bool vertex_main()
{
    return push_uint == 1;
}
