use fxprof_processed_profile::{
    Category, CategoryColor, CategoryHandle, Profile, SubcategoryHandle,
};

#[derive(Copy, Clone)]
pub enum CategoryOrSubcategory {
    Category(&'static str, CategoryColor),
    Subcategory(&'static str, CategoryColor, &'static str),
}

pub struct CategoryMatcher {
    matchers: Vec<(&'static [&'static str], SubcategoryHandle)>,
}

impl CategoryMatcher {
    pub fn new(
        profile: &mut Profile,
        matchers: &[(&'static [&'static str], CategoryOrSubcategory)],
    ) -> Self {
        let matchers = matchers
            .iter()
            .map(|(fragments, subcategory_or_category)| {
                let subcategory_handle = match *subcategory_or_category {
                    CategoryOrSubcategory::Category(name, color) => {
                        profile.handle_for_category(Category(name, color)).into()
                    }
                    CategoryOrSubcategory::Subcategory(name, color, subcategory_name) => {
                        let category = profile.handle_for_category(Category(name, color));
                        profile.handle_for_subcategory(category, subcategory_name)
                    }
                };
                (*fragments, subcategory_handle)
            })
            .collect();
        Self { matchers }
    }

    pub fn get(&self, fragments: &[&str]) -> SubcategoryHandle {
        for (matcher_fragment_sequence, matcher_category) in &self.matchers {
            for fragment_sequence in fragments.windows(matcher_fragment_sequence.len()) {
                if fragment_sequence == *matcher_fragment_sequence {
                    return *matcher_category;
                }
            }
        }
        CategoryHandle::OTHER.into()
    }
}
