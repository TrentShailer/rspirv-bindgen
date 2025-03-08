[vk::push_constant]
ConstantBuffer<Line> line;
struct Line
{
    float2 start;  // Offset 0
    float2 end;    // Offset 8
    float4 colour; // Offset 16
}

[vk::push_constant]
ConstantBuffer<uint> push_uint;

[Shader("compute")]
[NumThreads(1, 1, 1)]
bool main()
{
    return push_uint == 1 || line.start.x == 0;
}
