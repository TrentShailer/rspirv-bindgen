
[SpecializationConstant]
const float NAME_A = 1.0f;

[SpecializationConstant]
const uint NAME_C;

[SpecializationConstant]
const double NAME_D = 1.0;

[SpecializationConstant]
const double NAME_E;

[SpecializationConstant]
const uint8_t NAME_B = -1;

[Shader("compute")]
[NumThreads(1, 1, 1)]
bool main()
{
    return NAME_B == -1 || NAME_C == 0 || NAME_D == 1.0 || NAME_E == 1.0 || NAME_A == 1.0;
}
