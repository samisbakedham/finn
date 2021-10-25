#pragma once

// Copyright (c) 2018-2019 David Burkett
// Distributed under the MIT software license, see the accompanying
// file LICENSE or http://www.opensource.org/licenses/mit-license.php.

#include <vector>
#include <Crypto/Models/BigInteger.h>
#include <Core/Serialization/ByteBuffer.h>
#include <Core/Serialization/Serializer.h>
#include <Core/Models/TransactionInput.h>
#include <Core/Models/TransactionOutput.h>
#include <Core/Models/TransactionKernel.h>
#include <json/json.h>
#include <numeric>

////////////////////////////////////////
// TRANSACTION BODY
////////////////////////////////////////
class TransactionBody
{
public:
	//
	// Constructors
	//
	TransactionBody(std::vector<TransactionInput>&& inputs, std::vector<TransactionOutput>&& outputs, std::vector<TransactionKernel>&& kernels);
	TransactionBody(const TransactionBody& other) = default;
	TransactionBody(TransactionBody&& other) noexcept = default;
	TransactionBody() = default;

	static TransactionBody NoSort(
		std::vector<TransactionInput> inputs,
		std::vector<TransactionOutput> outputs,
		std::vector<TransactionKernel> kernels
	);

	//
	// Destructor
	//
	~TransactionBody() = default;

	//
	// Operators
	//
	TransactionBody& operator=(const TransactionBody& other) = default;
	TransactionBody& operator=(TransactionBody&& other) noexcept = default;

	//
	// Getters
	//
	const TransactionInput& GetInput(size_t idx) const
	{
		assert(idx < m_inputs.size());
		return m_inputs[idx];
	}
	const std::vector<TransactionInput>& GetInputs() const { return m_inputs; }

	const TransactionOutput& GetOutput(size_t idx) const
	{
		assert(idx < m_outputs.size());
		return m_outputs[idx];
	}
	const std::vector<TransactionOutput>& GetOutputs() const { return m_outputs; }

	const TransactionKernel& GetKernel(size_t idx) const
	{
		assert(idx < m_kernels.size());
		return m_kernels[idx];
	}
	const std::vector<TransactionKernel>& GetKernels() const { return m_kernels; }

	uint64_t CalcFee() const noexcept
	{
		return std::accumulate(
			m_kernels.cbegin(), m_kernels.cend(), (uint64_t)0,
			[](uint64_t sum, const TransactionKernel& kernel) { return sum + kernel.GetFee(); }
		);
	}

	uint8_t GetFeeShift() const noexcept
	{
		uint8_t fee_shift = 0;
		for (const auto& kernel : m_kernels) {
			if (kernel.GetFeeShift() > fee_shift) {
				fee_shift = kernel.GetFeeShift();
			}
		}

		return fee_shift;
	}

	uint64_t CalcWeight(const uint64_t block_height) const noexcept;

	//
	// Serialization/Deserialization
	//
	void Serialize(Serializer& serializer) const;
	static TransactionBody Deserialize(ByteBuffer& byteBuffer);
	Json::Value ToJSON() const;
	static TransactionBody FromJSON(const Json::Value& transactionBodyJSON);

private:
	// List of inputs spent by the transaction.
	std::vector<TransactionInput> m_inputs;

	// List of outputs the transaction produces.
	std::vector<TransactionOutput> m_outputs;

	// List of kernels that make up this transaction (usually a single kernel).
	std::vector<TransactionKernel> m_kernels;
};