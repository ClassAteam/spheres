This code might serve as a reference that shows how matrix transformation impacts cube visualization.

## Important Notes

Please note that the egui library (wrapped in a higher-level library to work with Vulkano directly) used in this project has very strict rules about rendering due to its 'immediate mode' nature. This means it has non-obvious design rules and requirements, such as the necessity to call `gui.draw()` every frame if it is initialized, due to the runtime coupling between vulkan_utils and vulkano_egui. 
