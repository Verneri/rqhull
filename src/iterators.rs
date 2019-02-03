use std::mem;
use crate::{vertexT, facetT, setT};

pub struct SetIter<'a,T>{
    facetp: Option<&'a mut *mut T>
}

impl<'a,T> SetIter<'a,T> {

    pub fn new(set:*mut setT) -> SetIter<'a, T> {
        if set.is_null() {
            SetIter{facetp: None}
        } else {
            unsafe {
                SetIter{facetp: Some(mem::transmute::<*mut *mut libc::c_void,&'a mut *mut T>(&mut (*set).e[0].p))}
            }
        }


    }




}

impl<'a, T> Iterator for SetIter<'a, T> {
    type Item = &'a mut T;



    fn next(&mut self) -> Option<&'a mut T> {
        match &mut self.facetp {
            None => None,
            Some(r) => {
                if !(*r).is_null() && !(**r).is_null() {
                    let facet = **r;
                    self.facetp = Some( unsafe{&mut *((*r as *mut *mut T).offset(1))});
                    Some(unsafe{&mut *facet})
                } else {
                    None
                }

            }

        }




    }
}


pub struct VertexIter<'a> {
    vertex: &'a mut vertexT
}


impl<'a> VertexIter<'a> {

    pub fn new(vtx: *mut vertexT) -> VertexIter<'a> {
        let vertex = unsafe {
            let vertex = vtx;
            if vertex.is_null() { panic!("null vertex encountered") };
            &mut *vertex
        };
        VertexIter{vertex}

    }
}


impl<'a> Iterator for VertexIter<'a> {
    type Item = &'a mut vertexT;

    fn next(&mut self) -> Option<&'a mut vertexT> {
        if self.vertex.next.is_null() {
            None
        } else {
            let result: Option<&'a mut vertexT> = unsafe { mem::transmute::<Option<&mut vertexT>, Option<&'a mut vertexT>>(Some(self.vertex))};
            self.vertex = unsafe {&mut *self.vertex.next};
            result
        }
    }
}


pub struct FacetIter<'a> {
    facet: &'a mut facetT
}


impl<'a> FacetIter<'a> {

    pub fn new(fct: *mut facetT) -> FacetIter<'a> {
        let facet = unsafe {
            let facet = fct;
            if facet.is_null() { panic!("null facet encountered") };
            &mut *facet
        };
        FacetIter{facet}

    }
}

impl<'a> Iterator for FacetIter<'a> {
    type Item = &'a mut facetT;



    fn next(&mut self) -> Option<&'a mut facetT> {
        if self.facet.next.is_null() {
            None
        } else {
            let result: Option<&'a mut facetT> = unsafe { mem::transmute::<Option<&mut facetT>, Option<&'a mut facetT>>(Some(self.facet))};
            self.facet = unsafe {&mut *self.facet.next};
            result
        }
    }
}

pub struct VecBuffer {
    current_length: usize,
    ptr: *mut [f64; 2],
    cap: usize,

}

impl VecBuffer {



    pub fn new(cap:usize) -> VecBuffer {

        let current_length = 0 as usize;
        let mut vv = Vec::<[f64;2]>::with_capacity(cap);
        let ptr = vv.as_mut_ptr();
        let cap = vv.capacity();
        mem::forget(vv);
        VecBuffer{current_length,ptr,cap}
    }

    pub fn write(&mut self, item: &[f64], place: u32) {
        if !self.ptr.is_null() {
            use std::cmp::max;
            let cl = max(place as usize, self.current_length);
            if cl >= self.cap {
                let mut tempv = unsafe { Vec::from_raw_parts(self.ptr, self.cap, self.cap) };
                tempv.reserve(cl + 1);
                self.ptr = tempv.as_mut_ptr();
                self.cap = tempv.capacity();
                mem::forget(tempv);
            }
            self.current_length = cl;
            let vertice = unsafe { self.ptr.offset((place - 1) as isize) } as *mut f64;
            for k in 0..2 {
                let coord = unsafe { *(item.as_ptr().offset(k as isize)) };
                unsafe { (*vertice.offset(k as isize)) = coord };
            }

        }

    }

    pub fn to_vec(mut self) -> Vec<[f64;2]> {
        let vec = unsafe { Vec::from_raw_parts(self.ptr, self.current_length, self.cap) };
        self.ptr = std::ptr::null_mut();
        vec
    }
}

impl Drop for VecBuffer {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            mem::drop(unsafe { Vec::from_raw_parts(self.ptr, self.current_length, self.cap)});
        }
    }
}


