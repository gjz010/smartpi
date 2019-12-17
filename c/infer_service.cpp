#include "memory"
#include "cstring"
#include "inference_engine.hpp"
#include "cstdint"
#include "string"
//#include "dlfcn.h"
#include "cassert"
#include "cstdio"
using namespace InferenceEngine;
#ifdef WIN32
#define EXPORTED extern "C" __declspec(dllexport)
#else
#define EXPORTED extern "C"
#endif
//std::string output_name;
EXPORTED void InitializeInferService(const char* nwpath, const char* wtpath, uint64_t use_cpu, void** core_ret, void** nn_ret){
	try{
		std::cout<<"[InferService] Bootstrapping..."<<nwpath<<" "<<wtpath<<std::endl;
		//Core core;
		std::unique_ptr<InferenceEngine::Core> core=std::make_unique<InferenceEngine::Core>();

		std::cout<<"[InferService] Core started."<<std::endl;
		CNNNetReader reader;
		reader.ReadNetwork(nwpath);
		reader.ReadWeights(wtpath);
		std::cout<<"[InferService] Reading network..."<<std::endl;
		auto network=reader.getNetwork();
		std::cout<<"[InferService] Network read..."<<std::endl;
		InferenceEngine::InputsDataMap input_info(network.getInputsInfo());
		InferenceEngine::OutputsDataMap output_info(network.getOutputsInfo());
		for(auto& item: input_info){
			auto input_data=item.second;
			input_data->setPrecision(Precision::U8);
			input_data->setLayout(Layout::NHWC);
			input_data->getPreProcess().setResizeAlgorithm(RESIZE_BILINEAR);
			input_data->getPreProcess().setColorFormat(ColorFormat::RGB);
		}
		for(auto& item: output_info){
			auto output_data=item.second;
			//output_name=item.first;
			output_data->setPrecision(Precision::FP32);
			output_data->setLayout(Layout::NC);
		}
		std::cout<<"[InferService] Creating network."<<std::endl;
		std::unique_ptr<ExecutableNetwork> nn=std::unique_ptr<ExecutableNetwork>(new ExecutableNetwork(std::move(core->LoadNetwork(network, use_cpu?"CPU":"MYRIAD"))));
		*core_ret=(void*)core.release();
		*nn_ret=(void*)nn.release();
		std::cout<<"[InferService] Bootstrapped!"<<std::endl;
	}catch(const std::exception& ex){
		std::cerr << ex.what() << std::endl;
	}
}

struct InferRequestWrapper{
    std::unique_ptr<InferRequest> req;
    std::string name;
	
};

EXPORTED InferRequestWrapper* InferBatch(ExecutableNetwork* nn, uint8_t* blobdata, size_t size, uint32_t batch_size, uint32_t channels, uint32_t height, uint32_t width){
	try{
		//std::cout<<"[InferService] Infering buffer["<<size<<"] ("<<batch_size<<","<<channels<<","<<height<<","<<width<<")"<<std::endl;
		auto infer_request=std::unique_ptr<InferRequest>(new InferRequest(std::move(nn->CreateInferRequest())));
		auto input_info=nn->GetInputsInfo();
		//std::cout<<"[InferService] InferRequest Created."<<std::endl;
		for(auto& item: input_info){
			auto input_name=item.first;
			InferenceEngine::TensorDesc tDesc(InferenceEngine::Precision::U8, {batch_size, channels, height, width}, InferenceEngine::Layout::NHWC);
			auto blob=InferenceEngine::make_shared_blob<uint8_t>(tDesc, blobdata, size);
			infer_request->SetBlob(input_name, blob);
		}
		//std::cout<<"[InferService] Start async..."<<std::endl;
		infer_request->StartAsync();
		//infer_request->Wait(IInferRequest::WaitMode::RESULT_READY);
		//std::cout<<"[InferService] Started!"<<std::endl;
		auto ret=std::make_unique<InferRequestWrapper>();
		ret->req=std::move(infer_request);
		for(auto& item: nn->GetOutputsInfo()){
			ret->name=item.first;
		}
		auto ptr=ret.release();
		//std::cout<<"[InferService] Returning wrapper "<<(uint64_t) ptr<<std::endl;
		return ptr;
	}catch(const std::exception& ex){
		std::cerr << ex.what() << std::endl;
		return 0;
	}
}
EXPORTED void PollInferResult(InferRequestWrapper* preq, float* ret, size_t size){
	try{
		if(preq==0) throw std::runtime_error("Bad");
		//std::cout<<"[InferService] Polling infer result for req "<<(uint64_t) preq<<std::endl;
		std::unique_ptr<InferRequestWrapper> req;
		req.reset(preq);
		//std::cout<<"[InferService] Wait..."<<std::endl;
		req->req->Wait(IInferRequest::WaitMode::RESULT_READY);
		//std::cout<<"[InferService] Waited!"<<std::endl;
		auto output=req->req->GetBlob(req->name);
		SizeVector dims = output->getTensorDesc().getDims();
		size_t input_rank = dims.size();
		//std::cout<<input_rank<<std::endl;
		auto const memLocker=output->cbuffer();
		const float* output_buffer=memLocker.as<const float*>();
		//std::cout<<"[InferService] memcpy "<<size<<std::endl;
		memcpy(ret, output_buffer, size*sizeof(float));
	}catch(const std::exception& ex){
		std::cerr << ex.what() << std::endl;
	}
}

