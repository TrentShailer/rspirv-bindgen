ParameterBlock<Descriptor1> descriptor1;
struct Descriptor1
{
    [vk_image_format("rgba16f")]
    RWTexture2D<float4> image;
    RWByteAddressBuffer maximum;
}

[vk_binding(0, 0)]
ParameterBlock<Descriptor2> descriptor2;
struct Descriptor2
{
    [format("rgba16f")]
    RWSampler2D<float4> sampler;
}

[vk_binding(1, 0)]
StructuredBuffer<float> descriptor3;

[shader("vertex")]
void vertexMain()
{
    uint width, height, levels;
    let dimentions = descriptor2.sampler.GetDimensions(0, width, height, levels);
    descriptor1.maximum.InterlockedAdd(0, width);
}

[shader("fragment")]
float fragmentMain()
{
    descriptor1.image.Store( { 0, 0 }, descriptor1.maximum.Load4(0));
    let index = 0;
    let value = descriptor3.Load(index);
    return value;
}
