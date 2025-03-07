
[SpecializationConstant]
const float NAME_A = 1.0f;

[SpecializationConstant]
const int8_t NAME_B = -1;

[SpecializationConstant]
const uint NAME_C;

[SpecializationConstant]
const double NAME_D = 1.0;

[SpecializationConstant]
const uint8_t NAME_E;

[Shader("compute")]
[NumThreads(1, 1, 1)]
bool main()
{
    return NAME_B == -1 || NAME_C == 0 || NAME_D == 1.0 || NAME_E == 0 || NAME_A == 1.0;
}
