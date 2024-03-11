use entities::api_req_params::{
    AssetSorting, GetAssetsByAuthority, GetAssetsByCreator, GetAssetsByGroup, GetAssetsByOwner,
    Pagination, SearchAssets,
};

pub trait RequestWithPagination {
    fn get_all_pagination_parameters(&self) -> Pagination;
    fn get_sort_parameter(&self) -> Option<AssetSorting>;
}

macro_rules! impl_request_with_pagination {
    ($struct_name:ident) => {
        impl RequestWithPagination for $struct_name {
            fn get_all_pagination_parameters(&self) -> Pagination {
                Pagination {
                    limit: self.limit,
                    page: self.page,
                    before: self.before.clone(),
                    after: self.after.clone(),
                    cursor: self.cursor.clone(),
                }
            }

            fn get_sort_parameter(&self) -> Option<AssetSorting> {
                self.sort_by.clone()
            }
        }
    };
}

impl_request_with_pagination!(GetAssetsByOwner);
impl_request_with_pagination!(GetAssetsByGroup);
impl_request_with_pagination!(GetAssetsByCreator);
impl_request_with_pagination!(GetAssetsByAuthority);
impl_request_with_pagination!(SearchAssets);
